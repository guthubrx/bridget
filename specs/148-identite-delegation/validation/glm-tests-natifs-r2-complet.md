# Rapport de ronde R2 — Bridget148 (testeur indépendant GLM)

Arbre figé : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/session-148-identite-delegation`
Branche `session-148-identite-delegation`, HEAD `6e167cb03f35c798b20ecc0c210ea6809df347e4`, WIP non commit (identique au début et à la fin de la ronde).
ENV : `umask 077`, `TMPDIR=/private/tmp/b148.oDC5Jq` (répertoire privé 0700, chemin court).
Aucune modification du repo, aucun Git, aucun restart, aucun modèle réel. Recette GLM réelle (opt-in) non exécutée, comme convenu.

## Verdict global

| Étape | Commande | PASS | FAIL | Ignored | RC | Durée |
|---|---|---|---|---|---|---|
| 1 | `cargo test -p bridget-daemon --test native_delegation_e2e native148_…` | 1 | 0 | 0 | **0** | 5,51 s |
| 2 | `cargo test -p bridget-daemon --test t3_session_identity_env` | 1 | 0 | 1 (volontaire) | **0** | 0,12 s |
| 3a | `cargo test --workspace --lib -- --test-threads=1` (umask 022 — ma erreur, voir incident) | 1196 | 2 | 14 | 101 | ~105 s |
| 3b | idem, ENV complet prescrit (umask 077) | **1496** | **0** | 15 | **0** | ~120 s |
| 4a | `cargo fmt --all -- --check` | — | **36 diffs** | — | 101 | — |
| 4b | `cargo clippy --workspace --lib -- -D warnings` | — | **4 erreurs** | — | 1 | — |

Logs : `01-native-e2e.log` … `10-clippy-lib.log` dans ce dossier.

## Étape 1 — e2e natif 148 : PASS

Le correctif R1 est vérifié par les assertions mêmes du test (toutes passées) :
- **un seul `started` et un seul `prompt`** avant redémarrage ; l'unique prompt porte `resume_card=true` ET `mission=true` (carte+mission jointes au premier tour du nouvel enfant natif).
- refus sans grant (`delegation_grant_required`), puis grant humain via CLI réel.
- idempotence : 10 replays ⇒ mêmes `task_id`/`child_agent_id`/`message_id`.
- réponse corrélée unique (`replies.len()==1`, `in_reply_to` = message d'origine, corps = réponse attendue).
- SQLite : 1 ligne `native_delegations` ; `t3_present=false` ; aucun répertoire `.t3` chez le fournisseur.
- **recette cancel/restart VRAIMENT atteinte** : mission `WAIT_CANCEL_148` bloquée chez le fournisseur factice, `Cancel` ⇒ `cancelled` avec résultat nul et aucune réponse corrélée ; `StopOrder` du premier enfant ; arrêt puis relance du daemon (2 générations) ; replay de la requête d'origine ⇒ résultat durable identique (`recovered == finished`) ; `cancelled` préservé après restart ; `started == 2` au total. Le test est une fonction unique sans sortie anticipée : son PASS prouve l'exécution de toutes ces assertions.

## Étape 2 — identité T3 : PASS

`partial_or_invalid_t3_proof_never_falls_back_to_pid` : les 4 cas (endpoint seul, authorization seule, paire invalide, paire vide) passent. Le helper ignoré `isolated_identity_entrypoint` écrit un marqueur PID réellement valide, résout l'identité native par le marqueur, et `bridget_who` répond `isError=true` + `t3_session_unavailable` — refus T3 malgré marqueur PID valide, sans repli PID. 1 ignored volontaire (helper).

## Étape 3 — suite workspace lib mono-thread

### Incident de ma ronde (consigné honnêtement)
J'ai appliqué `umask 077` uniquement à la création du répertoire privé, pas aux processus de test. Le run 3a a donc tourné sous umask 022 :
- `daemon::presence_tests::sigkill_daemon_reconcilie_l_ancien_groupe_avant_une_reprise_unique` — FAIL (« socket daemon absente après reprise »).
- `daemon::presence_tests::stop_apres_register_traverse_le_wrapper_et_le_superviseur_reels` — FAIL (« le wrapper réel ne s'est pas enregistré »).

**Diagnostic (données runtime)** : le daemon enfant du premier test panic à `daemon.rs:20871` avec `état non privé dans le namespace : /tmp/bg908-e2e-…/.state/bridget.db`. Ces deux tests codent leur racine en dur sous `/tmp` ; le fichier `bridget.db` créé par le processus de test prend le mode de l'umask. Sous 022 ⇒ 0644 ⇒ garde de vie privée du produit refuse ; sous 077 ⇒ 0600 ⇒ accepté. Isolement avec `umask 077` : les deux tests PASS (0,84 s / 0,28 s). Le run 3a s'est aussi arrêté en fail-fast : le binaire à 300 tests n'avait pas tourné.

**Classification : environnement (mon umask), pas produit.** Le run de référence 3b sous ENV complet prescrit est le seul compté.

### Run de référence 3b (umask 077 + TMPDIR privé, mono-thread)
- RC=0. Aucun échec. Plus aucun EPERM presence (les 8 échecs R1 sur `/tmp/bridget-*.fleet.json` ont disparu : le TMPDIR privé 0700 appartient au processus).
- Plus aucun échec SUN_LEN (chemin court).
- Comptes : 45 (1er crate) + **1151 / 0 / 14** (bridget-daemon lib, 97,77 s) + **300 / 0 / 1** (3e crate, 21,25 s) = **1496 PASS, 0 FAIL, 15 ignored**.
- L'isolation TMPDIR a fonctionné : ~1200 sous-répertoires et fichiers scratch (WAL sqlite) des suites atterrissent dans le répertoire privé, pas dans `/tmp` partagé. Nettoyés après conservation des logs.

## Étape 4 — fmt / clippy : FAIL (défauts WIP + 1 vieux warning de base)

### fmt (`--check`, RC=101) : 36 diffs, tous dans des fichiers du WIP
`wrapper.rs` (13), `registry.rs` (8), `daemon.rs` (5), `lib.rs` (2), `lifecycle.rs` (2), `protocol.rs` (3), `src/daemon/native_delegation.rs` (1), `tests/native_delegation_e2e.rs` (1), `tests/native_delegation_real_glm.rs` (1). Détail complet : `09-fmt-check.log`.

### clippy (`-D warnings`, RC=1) : 4 erreurs
1. `src/daemon/native_delegation.rs:451` — `cmp_owned` (WIP).
2. `src/daemon/native_delegation.rs:790` — `collapsible_if` (WIP).
3. `src/daemon/native_delegation.rs:855` — `collapsible_if` (WIP).
4. `src/store/threads.rs:728` — `too_many_arguments` (8/7) sur `read_range_ordered`. **Vieux warning de base** : fichier identique à HEAD (vérifié `git diff --quiet HEAD`), la signature à 8 paramètres existe déjà sur main.

## Empreintes SHA-256 (début = fin, `hashes_debut.txt` / `hashes_fin.txt`)

| Fichier | SHA-256 |
|---|---|
| `src/mcp.rs` | `bd7919cb6541f9c1021487e36f953315047369d5372e4716399e8ea2facb5cd7` |
| `src/wrapper.rs` | `8638b54fe5cf8a2644ef219bcc8f3adf5055c6a77c25d9971b2d1222a0ca0969` |
| `src/daemon/native_delegation.rs` | `cdeecabf5c2d2babfef2af9be32df9744ccfa66030e6fd52a67fea274e495f40` |
| `tests/native_delegation_e2e.rs` | `9b6ccfff062abedca1d85d6998c0f6dcea201967c4bab0a014c221fc32df64d6` |
| `tests/t3_session_identity_env.rs` | `8a924cdb9f60069430ef0bfdfb37d08ab7372b8ae9b488ba36369366d59a498d` |
| `tests/fixtures/native_delegation_148.py` | `3fb663cbdcaa2759053ee8900c00012c83409522bda300e5262af12b74e3f754` |
| `tests/fixtures/native_delegation_real_glm_relay.py` | `30c23c649728b72eac5eaab497e3d303d5597df98cb7dcaff175ba484a25787d` |

## Résidus

- **Aucun processus résiduel** de mes tests (worktree, daemons de test, fixtures python : rien). Les ~22 processus `bridget` présents (daemon production, `t3 serve`, MCP de sessions) préexistaient à la ronde — inchangés, non touchés.
- 4 répertoires `/tmp` issus de mes 2 exécutions en échec (umask 022) supprimés après inspection : `bg908-e2e-89339-b4e6777b`, `bg907-e2e-89339-64846808`, `bg908-e2e-4907-c9183c55`, `bg907-e2e-5470-925ab367`. Les autres répertoires `/tmp` (runs antérieurs du dev/R1) n'ont pas été touchés.
- Scratch TMPDIR privé nettoyé ; seuls les logs et empreintes sont conservés.

## Limites

1. Recette GLM réelle (modèle réel) non exécutée : opt-in non demandé cette ronde.
2. `cargo clippy` s'arrête à la première crate en échec (`bridget-daemon`) : d'autres warnings pourraient exister en aval après correction des 4 erreurs.
3. Le run 3a (umask 022) est conservé comme preuve de diagnostic, pas comme référence.

**Budget : ~20 min / 60 min.**
