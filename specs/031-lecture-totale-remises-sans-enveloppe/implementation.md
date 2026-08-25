# Preuves d'implémentation

## Références

- Parent empilé : `7ea339b789efe076d2a5d194067e431060f5ec6c`
- Base commune avec `origin/main` au démarrage :
  `b6eea777facf929d99a9c4f9ae75fb50e06dc2fd`
- Branche : `session-031-lecture-totale-remises-sans-enveloppe`
- Objectif : `cc47cfc5-a7d4-46f1-afe3-ece4ccbe864b`
- Délégation : `bd7c1d3f-aa61-42b4-84ea-0dbc76e2ca7c`

## Ordre de preuve

1. Compiler la tête parente avec `cargo test --no-run`.
2. Appliquer les deux oracles du jury et mesurer leurs rouges sans correctif.
3. Ajouter un oracle indépendant pour P1, P2 et P3.
4. Implémenter la reprise, puis l'accusé, dans des commits séparés.
5. Tuer séparément les trois mutants causaux et restaurer l'arbre après chacun.
6. Rejouer les mêmes commandes, puis les gates du workspace.

## Mesures

### Instrument

- Plateforme : Linux `6.8.0-94-generic`, x86_64, machine cartae.
- Charge moyenne sur une minute pendant la campagne ciblée : `0,83` avant et
  après. La suite workspace a démarré à `0,88 / 1,51 / 2,14`.
- Les filtres ont été vérifiés par `cargo test -p bridget-daemon --lib -- --list`
  avant comptage : 458 tests sur la tête sans les nouveaux oracles, et les deux
  noms du jury étaient présents.
- Tous les témoins SQLite ciblés ont été joués avec `--test-threads=1`.

### Rouge initial

`cargo test --no-run` compile la tête avant correctif.

- Jury : `0 passé / 2 échoués / 0 ignoré`. Le chemin d'accusé s'arrête sur
  `InvalidColumnType(5, message_bytes, Null)` ; celui de reprise sur
  `InvalidColumnType(4, message_bytes, Null)`.
- Métier : `0 passé / 3 échoués / 0 ignoré`. Les trois témoins s'arrêtent sur
  la même rigidité SQLite avant leur assertion discriminante. Ce compte prouve
  une seule cause commune, pas P1, P2 et P3.

### Mutants causaux

Chaque mutant conserve la projection nullable afin que le rouge tombe dans
l'assertion métier visée, et non dans le décodage SQLite.

- P1 — filtrage silencieux et reclassement neutralisé : les deux oracles du
  jury passent (`2 / 0 / 0`), mais P1 échoue (`0 / 1 / 0`) sur la requête SQL
  brute, `dispatching` au lieu de `indeterminate`. P2 rend d'abord les deux
  identifiants valides, puis échoue sur son second contrôle de phase.
- P2 — reclassement neutralisé sans filtrage : P2 échoue (`0 / 1 / 0`) sur
  `CorruptRecord("remise dispatching sans enveloppe après reclassement")`, sans
  `InvalidColumnType`, avant de rendre les remises valides.
- P3 — refus explicite d'un accusé lorsque l'enveloppe manque : P3 échoue
  (`0 / 1 / 0`) sur `Err(InvalidDelivery)` ; les témoins SQL voient encore
  `(dispatching, dispatching, NULL)` et aucune trace de corrélation impossible.

Après chaque mesure, le mutant a été retiré. La restauration finale compile et
rend le jury à `2 / 0 / 0` puis les propriétés métier à `3 / 0 / 0`, avec les
mêmes commandes et le même mono-thread.

### Gates de la tête restaurée

- `cargo fmt -p bridget-daemon -- --check` : vert.
- `cargo clippy -p bridget-daemon --all-targets -- -D warnings` : vert.
- `cargo test --no-run` : vert sur tout le workspace.
- `cargo test --no-fail-fast -- --quiet` : `936 passés / 2 échoués /
  16 ignorés` sur Linux. Les deux rouges sont préexistants et hors diff :
  `attach::tests::raw_mode_restaure_le_terminal_apres_eof_du_pseudo_tty`
  (`tcgetattr` rend EIO après fermeture du PTY sous Linux) et
  `lifecycle::tests::matrice_sc003_couvre_les_onze_familles_sans_residu_operationnel`
  (attente littérale sans `cursor`). Le rouge historique de reconnexion socket
  est passé pendant ce tir ; aucune stabilité n'en est déduite.
- `cargo fmt --all -- --check` : rouge hors périmètre dans quatre fichiers
  Maicie déjà non formatés (`routines.rs`, `store.rs`, `main.rs` et le contrat
  `routines.rs`). Aucun de ces fichiers n'a été reformatté par 031.
- Le gate `--features test-support` n'a pas été mesuré : il ne compile pas sur
  cette plateforme Linux à cause des appels `kqueue`/`kevent` non gardés dans
  `idempotency_crash_test.rs`. Il n'a pas été forcé.

## Composition mesurée

- Après le dernier fetch, `origin/main` vaut
  `4f04916de77a3688c46afc04828e3c573541df76` et `merge-tree` avec la tête 031
  est propre.
- `merge-tree` avec
  `origin/fix/purge-ne-doit-pas-orpheliner-en-silence` signale des conflits dans
  `daemon.rs` et `idempotency.rs`. Le même ensemble de fichiers est déjà en
  conflit contre le parent `7ea339b789efe076d2a5d194067e431060f5ec6c` : 031
  n'ajoute donc pas un nouveau conflit textuel.
- La résolution devra conserver les deux décisions distinctes : une reprise
  reclasse seulement `dispatching + NULL` en `indeterminate`, tandis qu'une
  purge reclasse seulement une remise encore `dispatching` en `orphaned`.
  Aucune des deux opérations ne doit écraser le classement terminal de l'autre ;
  un accusé sur `orphaned` garde son verdict métier sans erreur SQLite.
