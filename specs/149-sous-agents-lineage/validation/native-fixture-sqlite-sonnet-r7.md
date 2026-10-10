# Fixture SQLite 149 - diagnostic et correctif r7 (Sonnet 5.5 high)

Date : 2026-10-10. Aucun Git, aucune installation, aucun redémarrage, aucun modèle réel, aucun fichier de production touché.
Racine : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage`.
Environnement : `CARGO_TARGET_DIR=/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-target`, `CARGO_BUILD_JOBS=2`, `CARGO_INCREMENTAL=0`, `--offline`, `TMPDIR=/tmp/b149t-r7` (0700), `umask 077`.

## Verdict

- **Cause prouvée** : deux tests du fichier pouvaient recevoir la même racine `/tmp/bcap-<pid>-<nonce>`. Ils partageaient alors la même base SQLite, le même socket et le même `agents.json`.
- **Correctif** : fixture de test seulement. Un compteur atomique est ajouté au nom de la racine. Un contrôle positif relit l'état posé.
- **Résultat** : `capabilities_integration_test` 3 PASS. 800/800 exécutions isolées PASS, 600/600 PASS sous 6 boucles parallèles. `bridget-daemon` complet : **1478 PASS, 0 FAIL, 61 ignorés** (79 exécutables), sans `RUST_TEST_THREADS` (non nécessaire).
- Aucun bug de production. Aucune assertion affaiblie. Aucune attente ajoutée.

## Diagnostic (avant tout correctif)

Donnée r6 : `called Result::unwrap() on an Err value: SqliteFailure(DatabaseBusy, "database is locked")` à la ligne 151, dans `referent_control::set`, après `Store::open`. La suite du test finissait en 0,22 s : l'erreur est donc revenue **sans attendre** le `busy_timeout`.

Lecture du code :
- `root()` (fixture) nommait la racine avec `pid` + `SystemTime::now().as_nanos()`.
- Les 3 tests du fichier tournent en threads d'un même processus : même `pid`.
- `referent_control::set` ouvre une transaction différée (`unchecked_transaction`) : lecture puis `UPDATE`. En mode WAL, quand un autre écrivain a validé entre les deux, SQLite rend `SQLITE_BUSY` **tout de suite**, sans appeler le gestionnaire d'attente. Ce comportement est documenté dans `store.rs` (`ensure_wal`) et `referent_control.rs:139`.
- Il faut donc un second écrivain sur le même fichier. Aucun daemon n'est lancé à cet instant. Seul un autre test avec la même racine l'explique.

Mesures (hypothèse : l'horloge a une résolution d'une microseconde) :

| Mesure | Résultat |
|---|---|
| `as_nanos() % 1000` sur macOS | `0` : résolution µs |
| 3 threads lancés ensemble, 20 000 rondes, programme isolé `nonce.rs` | 3183 rondes avec nonce dupliqué (16 %) |
| Fichier original, 60 exécutions | 0 échec |
| Fichier original, 100 exécutions | 1 échec (`modele_non_declare…`, assertion ligne 242 : premier message reçu différent de `SpawnRejected`) |
| Fichier original + sonde temporaire de doublon de racine, 400 exécutions | 11 échecs, **11 sondes `DIAG-COLLISION-RACINE`** (2,75 %), aucun échec sans sonde |

Les 11 échecs coïncident exactement avec les 11 collisions de racine. Le même mécanisme produit deux symptômes : `DatabaseBusy` (r6, ligne 151) et message croisé (lignes 84 et 242, vus pendant le diagnostic). La sonde a été retirée avant le correctif.

Cause secondaire écartée : aucun daemon résiduel, aucun autre processus sur ces bases (`ps`, `lsof`). Le daemon de production PID 58394 n'a pas été touché.

## Changement

Fichier unique : `crates/bridget-daemon/tests/capabilities_integration_test.rs` (+18/-1). Diff : `native-fixture-sqlite-r7.diff`.
1. `static SEQUENCE: AtomicU64` ; la racine devient `/tmp/bcap-<pid>-<nonce>-<n>`. Unique par construction. Longueur du socket : environ 55 caractères, sous la limite `SUN_LEN`.
2. Contrôle positif dans `root()` : après `referent_control::set`, relecture avec `generation == initial + 1` et `agent_posture == Some(Complete)`. La fixture prouve son état initial avant toute demande.

Inchangés : les oracles zéro-spawn (`marker` absent), le refus `UnsupportedCapability`, l'ordre durable absent après refus, les durées, le contrat produit. `rustfmt --check` sur ce seul fichier : OK (aucun formatage global).

## Validation exacte

| Contrôle | Résultat |
|---|---|
| Fichier complet, `cargo test --test capabilities_integration_test` | 3 PASS |
| Binaire corrigé, 800 exécutions en série | 800 PASS, 0 FAIL |
| Binaire corrigé, 6 boucles parallèles x 100 | 600 PASS, 0 FAIL |
| Attendu sans correctif sur 1400 exécutions (2,75 %) | environ 38 échecs ; observé : 0 |
| `cargo test -p bridget-daemon --offline --no-fail-fast` (`native-r7-daemon.log`, sha `eddd434c…112a`) | **1478 PASS, 0 FAIL, 61 ignorés**, 79 exécutables, 6 min 20 |
| `bridget-transport` (r6, source inchangée, `native-r6-final-transport.log`) | 328 PASS, 0 FAIL, 2 ignorés |

Agrégat (deux exécutions distinctes, pas un seul passage) : **1806 PASS, 0 FAIL, 63 ignorés** = daemon r7 (1478/0/61) + transport r6 (328/0/2). Le total r6 était 1805/1/63 : +1 PASS = le test qui échouait.
Paramètre de concurrence : défaut du harnais. `RUST_TEST_THREADS=2` non utilisé, non nécessaire.

## Garde production et binaires

- Empreinte source production : `6cc9a2be1dd3e60d59e4035574ebbb767095fc235f0a691b8481125ceb1f367f`, 111 fichiers. **Identique à r6.**
- Empreinte complète (tests inclus) : `d44190ab30d1ed66861c7f10640507d921f87339a2298dd115edce8808574f3f`, 198 fichiers. r6 : `ebed6dc3…edcf`. Seule différence : le fichier de test. Son sha : `8e903026434673ff20abec746f3aeb0785cd8be215cfd083471435336bf8cd4e`.
- Aucun `.rs` de production plus récent que le binaire release. Aucun nouveau fichier source de production.
- Binaires stables non reconstruits, non copiés, shas relus : debug `620e729fca53c621…`, release `abfb346e23ccf51d…` (inchangés). `Cargo.toml` et `Cargo.lock` non modifiés par cette ronde.
- Nettoyage : 2 dossiers `/tmp/bcap-34525-*` laissés par des échecs de ma première série (sans `umask 077`), vérifiés sans processus (`lsof`), supprimés un par un. Aucun `kill`. Aucun daemon de test restant.

## Limites

- Non reproduit sous forte charge runtime globale ni avec verrous réels : reporté, comme demandé.
- Les 2,75 % mesurés viennent d'une machine peu chargée avec le binaire de test ; le taux r6 sous suite complète peut différer. La preuve est causale (11 échecs = 11 collisions), pas statistique.
- Le même motif `pid` + `as_nanos` existe dans d'autres fichiers de tests (`attach_journal_attestation_test.rs`, `execution_store_test.rs`, `daemon_shutdown_test.rs`, `spawn_refusal_hosts_test.rs`, `work_submission_test.rs`). Non modifiés (hors périmètre). Ils sont exposés seulement si deux tests du même fichier partagent une racine ; non vérifié.
- Observation pour Sol, sans action : `referent_control::set` démarre une transaction **différée** (lecture puis écriture). Face à deux écrivains simultanés, elle peut échouer sans attente. En production, aucune preuve de double écrivain ici ; non vérifié.
