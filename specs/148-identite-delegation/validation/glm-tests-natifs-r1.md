# Rapport de ronde R1 — Bridget148 (testeur indépendant)

## Verdict global

| Commande | PASS | FAIL | Ignored | RC |
|---|---|---|---|---|
| 1. `native_delegation_e2e` (test 148 unique) | 0 | **1** | 0 | 101 |
| 2. `t3_session_identity_env` (4 cas env) | 1 | 0 | 1 (volontaire) | 0 |
| 3. `cargo test --workspace --lib` (TMPDIR=/tmp) | 1184 | **11** | 14 | 101 |

Hashes début = fin (sources natives stables) :
- `native_delegation_e2e.rs` : `7d8f298cdd15…cf122913`
- `t3_session_identity_env.rs` : `8a924cdb9f60…d59a498d`

Aucun processus résiduel. Aucune fixture modifiée. Aucun fix tenté.

## Échec 1 — test e2e 148 : `left: 2, right: 1` (native_delegation_e2e.rs:345)

**Données réelles obtenues** (fixture conservée `/tmp/n148-dae866e7dd`) :
- `evidence.jsonl` : 1 `started` (pid 5171), **2** `prompt` (même pid).
- Journal enfant `state/sessions/…/2026-10-10.jsonl` : **2 tours distincts** — msg `e125b5522d054` (corps « Carte de reprise Bridget… ») puis msg `c8e6835d…` (corps « Inspecte les faits locaux de la fixture148. »).
- Log daemon : un seul `Send de conn-4`.

**Cause racine** : le wrapper enfant injecte la carte de reprise (`wrapper.rs:329`, `managed_resume_context`) comme premier tour à tout agent managé. Le provider factice compte chaque frame user. La mission compte donc 2 prompts. Les autres assertions tiennent : `started == 1`, réponse corrélée unique (`replies.len() == 1`), `result == ANSWER`, idempotence des 10 replays.

**Classification** : conflit de spécification, pas un défaut d'exécution. Soit le produit doit désactiver la carte pour un enfant de délégation native (un tour de provider réel serait consommé), soit le test doit compter la carte. Décision à prendre côté dev.
**Reproduction** : `cd …/.worktrees/session-148-identite-delegation && TMPDIR=/tmp cargo test -p bridget-daemon --test native_delegation_e2e -- native148_sans_t3_retry_reponse_correllee_annulation_et_reprise_durable -- --nocapture`

## Échec 2 — `mcp::tests::matrice_fr009` : `left: 24, right: 20`

**Cause prouvée par diff** (worktree vs principal) : le WIP route 4 nouveaux outils (`bridget_capabilities`, `bridget_delegate`, `bridget_task_status`, `bridget_task_cancel`) et étend `tools()` via `delegation_mcp::tools()` (mcp.rs:473, 2235). La matrice FR-009 attend toujours 20.
**Preuve croisée** : même test sur le checkout principal (base main) → PASS (20 outils).
**Classification** : défaut WIP (outils ajoutés, assertion de conformité pas mise à jour). Stable et reproductible.

## Échecs 3 — 8 × `daemon::presence_tests::*` : `EPERM` sur `/tmp/bridget-*.fleet.json`

**Classification** : environnement, pas produit. Preuve : le même test passe avec le TMPDIR par défaut (`/var/folders/…`) et échoue avec `TMPDIR=/tmp`. Contrainte contradictoire avec SUN_LEN (voir échec 5).

## Échecs 4 — `native148_glm_catalogue…` : `cwd_outside_parent_grant`

**Cause** : la fixture accorde le grant sur `std::env::temp_dir()` ; le test code `cwd: "/tmp"` en dur (native_delegation.rs:1569). Avec TMPDIR par défaut, `/tmp` est hors grant. Le refus est le comportement de sécurité voulu.
**Classification** : dépendance TMPDIR codée en dur dans le test WIP. Passe avec `TMPDIR=/tmp`.

## Échecs 5 — SUN_LEN : `path must be shorter than SUN_LEN`

`spec104_ancien_daemon`, `inventory_provenance` (×2), `matrice_roles`, `spec094_domaine` (×2), etc.
**Classification** : environnement macOS (limite socket 104). Ces tests échouent avec le TMPDIR par défaut et passent avec `TMPDIR=/tmp`. C'est la raison de l'imposition TMPDIR=/tmp.

## Flaky observés (interférences parallèles)

Sans TMPDIR=/tmp : 13 échecs dont ensemble différent. Isolément : `cli::idempotency_projection`, `spec094_inventaire`, `mcp::matrice` passent seuls. `desired_state::crash_after_rename` est apparu puis disparu entre runs. La composition des échecs varie entre runs complets → interférences entre tests parallèles, à traiter séparément.

## Limites

1. **Worktree vivant** : `native_delegation.rs` modifié à 06:08 puis 06:10 pendant ma ronde ; le compte lib est passé de 1157 à 1162 tests entre deux runs. Mes counts finaux datent de l'état 06:11.
2. **Erreur de ma part corrigée** : un `cd` vers le checkout principal a dévié des chemins relatifs. Deux conclusions intermédiaires invalides ont été écartées. Les counts et diffs finaux utilisent des chemins absolus.
3. La moitié cancel/restart du test e2e 148 n'a pas été atteinte (échec avant). La reprise durable reste non vérifiée par ce test.
4. Écart de contexte confirmé vs ronde précédente : le test env couvre le gap identité (4 cas partiels/invalides, jamais de repli PID). L'interop HTTP réelle reste hors périmètre, comme convenu.

**Budget** : ~50 min / 60 min.
