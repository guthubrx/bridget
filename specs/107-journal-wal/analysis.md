# Analyse 107 — Analyze et Converge (2026-09-18)

## Verdict

Analyze : aucun finding CRITICAL. Converge (passage 1) : `CONVERGED` — chaque exigence a une preuve
`fichier:ligne` et un test. Recette release n° 2 : 1513/0. Statut : **Implemented** (diff non commité).

## Findings Analyze

| # | Finding | Sévérité | Correction |
|---|---|---|---|
| A1 | Spec : « 9 scénarios » alors que les tests sont 4 unitaires + 1 d'intégration adapté + régressions | low | Fiche corrigée à la clôture (compte réel) |
| A2 | Plan : `BEGIN EXCLUSIVE` supposé suffisant pour un `SQLITE_BUSY` en WAL | medium (trouvé par test) | `locking_mode=EXCLUSIVE` exige qu'aucune autre connexion WAL ne soit ouverte : S14 referme les siennes avant ; consigné dans research/ADR |
| A3 | Clippy : emprunt inutile dans un test | low | Corrigé |
| A4 | Recette release n° 1 : `core_089_concurrency_test` — 8 ouvertures simultanées d'une base neuve → `DatabaseBusy` sur la bascule (pas de gestionnaire d'attente sur cette transition) | high (trouvé par recette) | `Store::ensure_wal` : relecture du mode + réessai borné 2 s ; rejoué ×3 vert |
| A5 | Recette release n° 1 : `core_089_migration_test` — l'en-tête d'une base future était modifié avant le refus de schéma | high (trouvé par recette) | pragma déplacé après `store_schema::validate` ; rejoué ×3 vert |
| A6 | Recette release n° 1 : `codex_app_server::tests::journal_codex_atteste…` (transport, sans lien avec WAL) | info | Vert ×3 isolé ; surveillé à la recette n° 2 |

Constitution : XVIII (aucune complexité ajoutée) ; XIX (un pragma, zéro helper, zéro dépendance ; pragma
non dupliqué dans les quatre autres ouvreurs car propriété persistante du fichier, justifié dans
research.md) ; XX (mesure avant/après sur le même banc : 96 747 → 0 refus ; hypothèse « delete bloque les
lecteurs » vérifiée par le test rouge avant correction).

## Converge — exigences → code → test

| Exigence | Preuve code | Test |
|---|---|---|
| FR-001 mode `wal` vérifié, erreur explicite sinon | `store.rs:Store::ensure_wal` | `spec107_base_neuve…` (`store/tests.rs:1975`) |
| FR-002 base mémoire acceptée | `ensure_wal` (`memory`) | idem |
| FR-003 conversion sans perte, idempotente | pragma persistant, aucune réécriture | `spec107_conversion…` (`:1985`) |
| FR-004 lecteur non bloqué (écriture ouverte et validation) | WAL (`ensure_wal`) | `spec107_lecteur…` (`:2053`) ; S14 (`search_104_test.rs:1014`) |
| FR-005 `synchronous` inchangé | aucun autre pragma (`rg synchronous` = 0) | ADR 041 |
| FR-006 lecteurs lecture seule existants | `Store::open_read_only` (`store.rs:96`), préflight `store_schema.rs:33` | S14, tests lib daemon (barrières READ_ONLY) : 935/0 |
| FR-007 `-wal`/`-shm` privés | droits hérités du fichier (SQLite) | `spec107_fichiers_auxiliaires…` (`:2029`) |
| FR-008 écrivain-écrivain borné | `busy_timeout` 2 s inchangé (`store.rs:78`) | `spec101_contended_snapshot…` (`store.rs:507`), `daemon.rs:9416` |
| FR-009 S14 adapté sans affaiblir l'assertion | `search_104_test.rs:1014-1040` | S14 |
| FR-010 documentation | `docs/demarrage-a-froid.md`, `README.md`, `scripts/deploy-remote.sh:71` | relecture |
| SC-001 | 0 `SQLITE_BUSY` / ≥ 200 lectures | `spec107_lecteur…` |
| SC-002 | 10 000 lignes, deux réouvertures | `spec107_conversion…` |
| SC-003 / SC-004 | recette 104 + release complète | voir implementation.md |
| SC-005 | 0600 | `spec107_fichiers_auxiliaires…` |

Ouvertures concurrentes d'une base neuve : `core_089_concurrency_test` (8 `Store::open` simultanés) ; base d'une
version future intacte avant refus : `core_089_migration_test`.

## Limites

- La conversion demande qu'aucune autre connexion n'ait de transaction ouverte au démarrage : le préflight
  `validate_existing` referme la sienne avant `Store::open` (`daemon.rs:4294` puis `:2919`).
- Une copie à chaud de `bridget.db` seul est incomplète (documenté).

## Audit court (Phase 7, manuel)

- Écarts spec/plan/code : aucun après A4/A5 ; le plan disait « après `busy_timeout` et avant la validation de
  schéma », le code place le pragma après la validation (corrigé dans le contrat : la validation est en lecture).
- Tests manquants : aucun exigé ; le comportement sous coupure de courant (`synchronous`) n'est pas testable ici
  et n'a pas changé.
- Minimalisme & frugalité : +1 fonction privée (`ensure_wal`, 30 lignes, dont la moitié de commentaires justifiant
  le réessai), 0 helper partagé, 0 dépendance. Potentiel minimalisme : ~0 ligne suppressible à comportement constant.
- Vertus LLM & responsabilité future : charge cognitive future = une règle documentée dans l'ADR 041 et le
  commentaire de `Store::open` ; hypothèses vérifiées par tests rouges puis verts et par la recette complète.
- Risque résiduel : copie à chaud incomplète (documenté) ; aucune sauvegarde automatique n'existe dans le projet.
