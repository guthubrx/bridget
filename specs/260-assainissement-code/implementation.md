# Journal 260 - mise en oeuvre

Statut du programme : In Progress. Règles : `contracts/regles-implementeur.md`. Tâches : `tasks.md`.

## Lots

| Lot | État | Remarque |
|-----|------|----------|
| 0 | intégré | T001 à T006 faits |
| 1 | intégré | T008 à T023 faits ; T024 différé (D16) |
| 2 | intégré | T025 à T030 faits |
| 3 | intégré | T031 à T035 faits |
| 4 | intégré | T036 à T053 faits (desired_state.rs arrêté) |
| 5 | intégré | T054 à T068 faits |
| 6 | intégré | T069 à T080 faits (158 blocs en repli) |
| 7 | intégré | T081 à T091 faits (T090 différée D7) |
| 8 | partiel | T092 à T095, T097, T098 faits ; T096 non faite |
| 9 | partiel | T099, T100, T102, T103, T112, T113 faits ; T101, T104 arrêtées ; T105 à T111 non faites |
| 12 | intégré | T125 à T127 faits |
| 13 | intégré | T128 à T136 faits ; T137 partiel (job CI différé) |
| 14 | intégré | T138 à T140 faits |
| 15 | intégré | T141, T142 faits |
| 17 | intégré | T151 à T154 faits |

## Journal

### T001 - Journal et règles
- Statut : fait
- Commit : voir l'historique du lot 0
- Fichiers : `specs/260-assainissement-code/implementation.md`
- Vérifications : règles et `decisions.md` lus depuis `origin/main`. Toutes les portes sont `en attente`.
- Écarts : aucun
- Non vérifié : rien

### T002 - Outils et disque
- Statut : fait (constat), avec réserve bloquante pour Cargo
- Fichiers : aucun
- Vérifications : `rg` 15.2.0, `python3` 3.12.12, `gh` 2.103.2, `ruff` 0.14.13, `git` 2.56.0, `gitleaks` 8.30.1, `scripts/build.py` présent. Absents : `shellcheck`, `actionlint`, `cargo-audit`.
- Écarts : `C-DISQUE` (`df -g /`) donne 14, 12 puis 11 à 12 Gio libres sur la durée de la séance, valeur qui bouge avec l'activité de la machine. Le seuil de la règle R4 est 12 Gio.
- Non vérifié : combien de Gio une première compilation complète consomme (le dossier `target` du dépôt principal pèse 4,8 Mio).

### T006 - Ligne de base du format
- Statut : fait
- Fichiers : `specs/260-assainissement-code/validation/base-format.txt`
- Vérifications : `cargo fmt --all -- --check` (ne compile pas) -> 120 fichiers signalés. Aucun fichier source modifié.
- Écarts : chemins ramenés en relatif.
- Non vérifié : rien

### T003, T004, T005 - Lignes de base des tests et de l'analyse
- Statut : différé (disque)
- Raison : ces tâches compilent tout le workspace. Le disque libre (11 à 14 Gio) est au seuil d'arrêt de R4 (12 Gio). Décision demandée à François : relever l'autorisation ou libérer de la place.

## Idées

### Lot 2 - Dépôt sans résidus (T025 à T030)
- Statut : fait (T025 à T030)
- Commits : `git log --oneline origin/main..` du lot : archivage des 15 résidus, `.gitignore` (ignorer les résidus), `.gitignore` (4 exceptions D3), inventaire.
- Fichiers : `specs/archives/residus-racine-20261110/` (15 fichiers déplacés par `git mv`), `.gitignore`, `validation/lot02-constat.txt`, `validation/lot02-gros-fichiers.txt`.
- Vérifications : constat de départ 345 fichiers suivis mais ignorés, 15 résidus de racine. Après : 0 résidu suivi à la racine, 15 dans l'archive ; `git ls-files -ci --exclude-standard` reste à 345 (D3 : on documente, on ne retire pas). Aucun code ou script ne lit les résidus (`rg` hors `specs/` : aucune ligne). `gitleaks` ciblé sur les commits du lot : aucune fuite. `python3 scripts/check-docs.py` : PASS (32 pages, 522 liens, 13 paires).
- Écarts : D3 et D5 `en attente` : actions par défaut appliquées (ne rien retirer, archiver sans supprimer). Le contrôle `scripts/check-secrets.sh` complet (historique entier) n'a pas été relancé : il a été arrêté par le système lors de la séance précédente ; remplacé par un scan ciblé.
- Gros fichiers (propositions, rien déplacé) : 11 fichiers de plus de 1 Mio. `daemon.rs` et `native_delegation.rs` (code vivant) : à alléger par les lots 4 et 5. Les deux fichiers `preuves.html`, trois `.patch` de `specs/` et `audits/`, `docs/adapters/t3/...` : à garder (preuves historiques) ou à sortir de Git sur décision de François.
- Non vérifié : rien d'exécutable n'est touché.

### Bilan lot 2
Pas d'activation : aucun fichier exécutable modifié.

### Lot 3 - Consignes allégées (T031 à T035)
- Statut : fait
- Commits : copie du bloc (`289c9327`), remontée des consignes (`7cc2326b`), retrait du bloc (`d87deac6`) sur la branche du lot.
- Fichiers : `AGENTS.md` (126 112 -> 10 144 octets), `specs/recus-de-session.md` (nouveau, 117 229 octets), `validation/lot03-constat.txt`.
- Vérifications : bornes du bloc D=112 et F=1087 relevées ; bloc copié octet pour octet (`diff` vide, 975 lignes) ; retrait par motif : 975 lignes retirées, 0 ajoutée par le motif ; taille 10 144 octets (moins de 20 480, SC04). `check-docs.py` : PASS. `gitleaks` ciblé : aucune fuite.
- Écarts : les huit consignes vivantes ont été écrites avec leur source. Le repère du registre Mac est `2b4f409e...f9d32c9` (lu dans le reçu 256, copié depuis le fichier, pas de mémoire). Pour le Python 3.9.6, le reçu 224 est cité.
- Non vérifié : si d'autres consignes vivantes existent dans les reçus au-delà des huit demandées par le guide (le guide n'en demandait pas davantage).

### Bilan lot 3
Pas d'activation : documentation seulement.

### Lot 12 - Stockage documenté (T125 à T127)
- Statut : fait
- Commits : `6348d671` (inventaire), `bd37ae63` (ADR proposé) sur la branche du lot.
- Fichiers : `specs/260-assainissement-code/validation/lot12-inventaire-stockage.md`, `docs/decisions/260-stockage-versions-schema.md` (statut Proposé).
- Vérifications : relevé par `rg` hors fichiers de tests ; `python3 scripts/check-docs.py` PASS ; `git diff origin/main --stat -- crates` vide (aucun code touché).
- Écarts : l'inventaire contient des cellules « non lu » (comportement face à une base plus récente pour trois mécanismes ; modes de journal de six magasins). Elles ne sont pas inventées.
- Non vérifié : ce qui est marqué « non lu » dans l'inventaire.

### Bilan lot 12
Pas d'activation : documentation seulement. Décision à prendre par François : ADR `260-stockage-versions-schema.md`.

### Lot 15 - Proposition de signature (T141 et T142)
- Statut : fait
- Commit : `5a92f1b4` sur la branche du lot.
- Fichiers : `specs/260-assainissement-code/validation/lot15-proposition-signature.md`.
- Vérifications : `check-docs.py` PASS ; `git diff origin/main --stat -- . ':!specs'` vide (rien hors `specs/`) ; D8 reste `en attente` (non modifiée).
- Écarts : la procédure de publication n'est pas lisible dans le dépôt (le script `publier.py` cité par la mémoire de la séance est absent). Écrit « non lue » dans la proposition.
- Non vérifié : la faisabilité de l'option C sur le flux de publication actuel.

### Bilan lot 15
Pas d'activation : documentation seulement. Décision D8 demandée à François.

### Lot 13 - Scripts contrôlés (T128 à T136 faits, T137 partiel)
- Statut : T128 à T136 faits ; T137 partiel (cible locale faite, job CI différé)
- Commits : sur la branche du lot : `ruff.toml`, imports inutiles et chaînage des exceptions, avertissements de boucle, exceptions de sécurité justifiées et documentation de l'`exec`, commentaires `set -e`, cible `lint-scripts`.
- Fichiers : `ruff.toml` (nouveau), `Makefile`, `scripts/*.py` (prepare-project, publish-android, real_hook_profile171, sentinel, update, tests/...), 4 scripts shell (commentaires), `validation/lot13-*.txt`.
- Vérifications : `ruff check scripts` : 137 signalements au départ (`lot13-ruff-avant.txt`) -> 0 ; `make lint-scripts` code 0 ; `bash -n scripts/*.sh` code 0 ; `py_compile` des fichiers modifiés OK (dossiers `__pycache__` supprimés) ; `scripts/tests` : `test_install` 44 tests OK, `test_sentinel` 14 OK, `test_publish_android` 7 OK, `test_prepare_project_view` 3 OK ; `test_updates` : 16 échecs sur 26, **identiques sur `origin/main`** (chemin `/private/var` contre `/var` sous macOS), donc hérités.
- Écarts :
  - Le guide attendait 6 scripts sans `set -e`. Lecture : `install-k1.sh` et `registre-nature-reclasser.sh` ont déjà `set -euo pipefail` ; `federate-ssh.sh` l'active dans `federation_main()` (le fichier est sourcé). Restent 4 scripts, tous documentés par un commentaire. **T135 adapté** : les deux scripts de test ne reçoivent pas `set -eu`, car ils comptent leurs échecs (variables `passed`/`failed`) ; un `set -e` changerait leur comportement. Ajout du seul commentaire.
  - B905 : avec `target-version = "py39"` ruff ne le signale plus (rien à faire).
  - Le fichier `scripts/sentinel.py` est embarqué dans le démon par `include_str!` : le commentaire d'invariant ne change pas le comportement mais change les octets du binaire à la prochaine compilation.
  - D11 `en attente` : action par défaut appliquée (mécanisme `exec` conservé, commentaire d'invariant et `# noqa: S102`).
- Non vérifié : le job CI `scripts` de T137 (`rust.yml` n'existe pas avant le lot 1) ; aucun script vivant n'a été exécuté.

### Bilan lot 13
Activation en attente : `scripts/sentinel.py` est embarqué dans le binaire du démon (octets modifiés, comportement identique). Les autres scripts sont distribués tels quels.

## Converge et audit manuel (état au 10 novembre 2026, fin de séance)

### Converge - périmètre livré (lots 0 partiel, 2, 3, 12, 13, 15)
Issue : CONVERGED sur ce périmètre. Aucune tâche ajoutée à `tasks.md`.

| Exigence | Preuve lue dans le code réel |
|----------|------------------------------|
| FR06, FR07 | `git ls-files` : 0 résidu suivi à la racine, 15 dans `specs/archives/residus-racine-20261110/` ; `.gitignore` documente les 4 exceptions et ignore `/.wip-*`, `/watch_*` |
| FR08, FR09 | `AGENTS.md` = 10 144 octets ; le bloc de `specs/recus-de-session.md` est identique à celui de `4947bea9` (`diff` vide) |
| FR22 | `validation/lot12-inventaire-stockage.md`, `docs/decisions/260-stockage-versions-schema.md` (Proposé) |
| FR23 | `ruff check scripts` : 0 ; `bash -n scripts/*.sh` : 0 erreur ; `make lint-scripts` : code 0. **Manque** : job CI (T137 partiel) |
| FR25 | `validation/lot15-proposition-signature.md` ; D8 inchangée |

Non convergé (non implémenté) : FR01 à FR05, FR10 à FR21, FR24, FR26, FR27, FR29 (lignes de base), et la partie CI de FR23.

### Audit manuel (le protocole V15 n'a pas été lancé)
- Écarts spec/plan/tâches/code : T137 coché partiel (CI) ; 5 lots à code uniquement documentaire ou script livrés ; 11 lots non commencés.
- Tests manquants : lignes de base des tests (T003 à T005) absentes ; aucune suite Rust lancée dans cette séance.
- Risques : `scripts/sentinel.py` est embarqué dans le binaire du démon (`include_str!`) : prochain build = octets différents, comportement identique ; `test_updates.py` a 16 échecs hérités (identiques sur `origin/main`).
- Contrôle de secrets complet (`scripts/check-secrets.sh`, historique entier) : arrêté par le système lors du premier essai ; remplacé par `gitleaks` ciblé sur chaque lot (aucune fuite).
- Contre-revue adverse : `bridget who` -> « Aucun agent connecté ».

### Bloc : disque
Le disque libre oscille entre 10 et 14 Gio (11 Gio à la dernière mesure). La règle R4 impose un arrêt complet sous 12 Gio. Le dossier `target` de ce dépôt ne fait que 4,8 Mio : la première compilation du workspace écrirait plusieurs Gio. Tous les lots qui compilent sont donc arrêtés avant T003. Première tâche à reprendre : **T002** (contrôle du disque), puis **T003**.

### T002 / T003 - reprise après libération du disque (10 novembre 2026)
- Constat : François a libéré de l'espace (23 Gio libres) et répondu D1, D9, D10 (voir `decisions.md`). Un premier essai de T003 a fait passer le disque de 22 à 7 Gio en 2 minutes (cache de dépendances de 6,8 Gio) ; le worktree a ensuite été retiré (fusion de la branche par une autre session) et le cache purgé par l'entretien de `scripts/build.py` avant la fin.
- Mesures de prudence pour la reprise : cache de build hors du dépôt (`$HOME/.cache/bridget-build-260`), variables `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0` et `CARGO_INCREMENTAL=0` (moins d'octets écrits ; le comportement des tests ne change pas). Le disque est relevé pendant le build.

### T002 - Outils et disque (clos)
- Statut : fait. Disque : 23 à 47 Gio libres après libération par François.

### T003 - Ligne de base des tests
- Statut : fait
- Fichiers : `validation/base-echecs.txt` (24 lignes), `validation/base-instables.txt` ; sorties complètes non commitées (`base-tests.txt`, `base2-tests.txt`).
- Vérifications : deux passages complets (`--test-threads=4`, environnement privé, `--offline --locked`) : 3714 passés, 24 échoués, 110 ignorés au premier passage ; 24 échecs identiques au second passage plus 1 échec propre au second (`spec185_workspace_repair_uncertain_stop_...`, noté instable). La liste de base est l'intersection des deux passages.
- Écarts : la base Mac historique (main `c48c30fb`) comptait 40 échecs ; elle en compte 24 ici sur `ca1e203c`.
  Variables ajoutées par prudence disque : `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, cache de build `$HOME/.cache/bridget-build-260`. Elles ne changent pas le comportement des tests.
- Non vérifié : la base Linux (T021).

### T004 - Ligne de base des noms de tests
- Statut : fait
- Fichiers : `validation/base-noms.txt` (3841 noms triés ; les noms ne portent pas de préfixe de crate).

### T005 - Ligne de base de l'analyse
- Statut : fait
- Fichiers : `validation/base-clippy.txt`
- Vérifications : `clippy --all-targets` avec `test-support` : échec de compilation du lot de tests du démon (3 erreurs, comme attendu : `never_loop`, `non_octal_unix_permissions`, `reversed_empty_ranges`) ; 398 lignes `warning`.
- Écarts : le total d'avertissements diffère de 385 (compte avec `test-support` et en comptant les lignes de résumé).
- Non vérifié : rien.

### Bilan lot 0
Lot 0 complet sauf T007 (cette intégration). Pas d'activation.

### Changement de méthode (10 novembre 2026, soir)
- `AGENTS.md` (commit `1d50c0fa`) impose désormais de travailler sur `main` dans le checkout principal, sans branche ni worktree. Les règles `contracts/regles-implementeur.md` (R0) en tiennent compte. Les fichiers de preuve de la ligne de base, qui étaient dans un worktree retiré par une autre session, ont été récupérés depuis `validation/reprise-nettoyage-20261110/` (manifeste SHA conservé là).

### Lot 1 - Filet automatique (T008 à T023 faits, T024 différé)
- Statut : T008 à T023 faits ; T024 différé (porte D16)
- Commits : sur `main` (dernier : `fix(daemon): Retablir la priorite des conditions reecrites par l'analyse`). `Makefile` (cibles `fmt`, `fmt-check`, `clippy`, `lint`), `Cargo.toml` (lints du workspace), `.github/workflows/rust.yml`, `.github/workflows/rust-audit.yml`, corrections de code par règle, `docs/build-cache*.md`.
- Vérifications :
  - `C-CHECK` code 0 ; analyse `clippy --all-targets -D warnings` : 3 erreurs et environ 400 lignes d'avertissements au départ (`base-clippy.txt`) -> 0 (`lot01-t019-clippy.txt`).
  - Porte : `C-TESTS` complet puis `C-ECHECS` : 24 échecs, **identiques** à `base-echecs.txt` (aucun nouveau, aucun disparu) (`lot01-echecs.txt`).
  - Workflow `rust.yml` : accepté par GitHub (trois passages réels lus avec `gh run list`) ; `rust-audit.yml` : lancé manuellement (`gh workflow run`).
  - Base Linux (T021) : trois passages du job `tests` ; `base-linux-echecs.txt` = 21 tests en échec dans les trois passages ; `base-linux-instables.txt` = 5 tests en échec dans un ou deux passages seulement.
- Écarts :
  - **Défaut trouvé par les tests et corrigé.** Les suggestions de `nonminimal_bool` appliquées automatiquement ont perdu des parenthèses à deux endroits : la commande Docker en lecture seule (`native_delegation.rs`) et une boucle `while` de `native_permissions149_e2e.rs`. Trois tests `spec241_*` ont échoué sur la première porte ; corrigés (parenthèses rétablies), `spec241` rejoué (25 tests OK) puis suite complète rejouée. Les autres réécritures booléennes (git, cargo, lifecycle, guidance, mobile, remote_worker) ont été relues : équivalentes (lois de De Morgan).
  - Deux suggestions de clippy qui cassaient la compilation ont été annulées ou adaptées (`redundant_closure_call` dans `specialists222.rs` ; `cmp_owned` remplacé par `Path::new` ; `div_ceil` avec `8usize`).
  - Plusieurs règles ont été appliquées en un commit groupé (suggestions vérifiées) au lieu d'un commit par règle ; les cas non automatisables ont reçu un `#[allow]` local avec commentaire (assertions de constantes, `zombie_processes` des tests, `unnecessary_unwrap`, `redundant_closure_call`, `items_after_test_module`, `useless_conversion` de `time_t`, `field_reassign_with_default`).
  - Variables de build : voir T003 (débogage réduit, cache hors dépôt).
  - Le workflow `format` et `tests` restent en observation (`continue-on-error`) : le format attend le lot 10, les tests attendent D16.
  - La cible `make lint` inclut `fmt-check`, qui échoue tant que le lot 10 n'est pas fait (120 fichiers hors format).
- Non vérifié : le premier passage réel de `rust-audit.yml` (résultat non lu) ; la décision D16.

### Bilan lot 1
Activation : D1 = sources seules. Code exécutable modifié (dont `native_lineage.rs`, `native_delegation.rs`, nombreuses réécritures de forme) : **activation en attente**.

### Lot 4 - Tests hors des fichiers de production (T036 à T053)
- Statut : fait. 16 fichiers de la liste du guide + 47 autres fichiers (« module à sortir absent de la liste est traité aussi »). Un fichier arrêté : `desired_state.rs`.
- Commits : un par fichier, `refactor(tests): Sortir les tests de <fichier>` (63 commits sur `main`), exécutés par `outils/extraire_module_tests.py` avec `C-CHECK` complet (compilation des tests) avant chaque commit.
- Fichiers : 63 fichiers de production allégés et autant de fichiers `*_tests.rs` ou `<fichier>_<module>.rs` voisins ; `validation/lot04-echecs.txt`, `lot04-noms.txt`, `lot04-restant.txt`.
- Vérifications : porte sur un cliché isolé de `HEAD` (copie hors dépôt, pour ne pas mêler le travail non commité d'une autre session) :
  - `C-ECHECS` : 24 échecs de la base, tous présents ; 1 de plus, `reprise_daemon_redelivre_les_octets_immuables_a_la_meme_instance`, rejoué seul 3 fois : échec, succès, succès (test d'intégration sensible à la charge, déjà cité dans `releves.md` section 15) -> noté dans `base-instables.txt`.
  - `C-NOMS` : liste identique à la base, plus 3 tests ajoutés par une autre session (`spec261_*`, commits arrivés sur `main` pendant le lot). La base de noms est actualisée avec ces 3 lignes (règle « Ligne de base périmée »).
  - Tests en ligne restants : 9 340 lignes (SC05 : moins de 15 000).
- Écarts :
  - `desired_state.rs` (module `tests`, 381 lignes) : l'outil a coupé trop tôt, car un littéral multi-ligne contient une ligne `}` en colonne 0 ; la compilation a refusé et la modification a été annulée. Le module reste en ligne (tâche arrêtée pour ce seul fichier). Amélioration de l'outil à envisager : contrôle d'équilibre des accolades.
  - `agent_profile.rs` et `agent_profile/specialists.rs` : l'outil refuse (un attribut `#[allow(...)] // commentaire` ajouté au lot 1 n'est pas reconnu) ; leurs modules restent en ligne.
  - Une autre session travaillait en parallèle dans le même arbre (quota Codex, `specs/261-quota-reprise`) ; ses modifications non commitées ont fait échouer deux extractions, rejouées avec succès ensuite. Mes commits n'ajoutent que mes chemins.
- Non vérifié : rien de plus.

### Bilan lot 4
Code exécutable : seuls des déplacements de tests. « Activation en attente » : sans effet fonctionnel.

### Lot 14 - Tests ignorés expliqués (T138 à T140)
- Statut : fait
- Commit : `docs(tests): Ajouter le motif des tests ignores et exiger un motif` ; `Cargo.toml` : lint `ignore_without_reason = "warn"` (bloquant par `-D warnings`).
- Fichiers : `managed_process_tests.rs` (4), `workspace_commit_view_tests.rs`, `daemon_presence_tests.rs`, `daemon.rs` (2) ; `validation/lot14-sans-motif.txt`.
- Vérifications : 8 `#[ignore]` sans motif trouvés (D14 : motif seulement, aucun test exécuté) ; chaque test est un enfant lancé par un autre test et gardé par une variable d'environnement (nom indiqué dans le motif). Contre-épreuve : un `#[ignore]` nu fait échouer l'analyse (`#[ignore] without reason`). Analyse `-D warnings` : code 0 après remise en état.
- Écarts : aucun. Le total de tests ignorés reste 66 (aucun test réactivé).
- Non vérifié : `C-NOMS` non relancé (un attribut de texte ne change pas les noms ; la compilation et l'analyse des tests sont passées).

### Bilan lot 14
Code de test seulement. Pas d'activation nécessaire.

### Lot 17 - Code mort (T151 à T154)
- Statut : fait
- Commit : `refactor(daemon): Traiter les attributs dead_code et garder le code reserve avec une raison` ; constat dans `validation/lot17-avant.txt`.
- Fichiers : `provider_accounts_tests.rs`, `remote_worker_tests.rs` (2), `native_permissions.rs`, `workspace_git.rs`, `daemon/native_delegation.rs`, `attach.rs` (3).
- Vérifications : compilation (lib et tests) et `clippy -D warnings` : code 0. Trois attributs étaient inutiles (attach.rs lignes 556 et 702 ; le compilateur signalait l'attente non remplie) et ont été retirés. `attach.rs:564` garde un `#[expect(dead_code, reason = ...)]` « boucle T806b, conservé par décision D12 ». Les autres reçoivent `#[expect]` (tests) ou `#[cfg_attr(not(test), expect(...))]` (code utilisé seulement par les tests) avec une raison. Rien n'est supprimé.
- Écarts : le constat contient 21 occurrences, pas 9 : 9 dans `src/` (traitées) et 12 dans `crates/bridget-daemon/tests/` (helpers partagés entre binaires de tests d'intégration : l'attribut y est l'idiome normal, laissé tel quel). `C-TESTS` complet non relancé pour ce lot (seuls des attributs de lint changent ; la compilation des tests est vérifiée) : il sera rejoué à la prochaine porte de lot.
- Non vérifié : `C-ECHECS` et `C-NOMS` du lot.

### Bilan lot 17
Attributs de lint seulement. « Activation en attente » : sans effet fonctionnel.

### Lot 6 - `unsafe` justifiés (T069 à T080)
- Statut : fait. Commentaires de repli : 158 blocs.
- Commits : fonction `effective_uid()` (`fsutil.rs`) et son test ; remplacement de 76 lectures `geteuid` dans les deux crates ; commentaires `SAFETY` sur 487 blocs ; lint `undocumented_unsafe_blocks = "warn"` (bloquant par `-D warnings`).
- Fichiers : 101 fichiers Rust, `Cargo.toml`, `outils/commenter_unsafe.py` (nouvel outil, ajoute un commentaire seulement quand la syntaxe du bloc permet de l'affirmer).
- Vérifications : `rg geteuid` : plus qu'une occurrence (la définition). Compilation et analyse `-D warnings` : code 0, aucun bloc `unsafe` sans commentaire (487 blocs signalés au départ, 0 à la fin, tests et intégration inclus). Porte sur cliché isolé de `HEAD` : `C-ECHECS` = base + 2 tests instables déjà connus (`reprise_daemon_redelivre...`, `spec185_workspace_repair...`) ; `C-NOMS` = base + exactement 1 ligne (le test de `effective_uid`). La base de noms est actualisée.
- Écarts :
  - Le constat compte 487 blocs (et non 247) parce que les tests déplacés au lot 4 sont désormais comptés et que clippy compte aussi les blocs de tests.
  - Un commentaire `// SAFETY :` existant (espace avant les deux-points) n'était pas reconnu : corrigé en `// SAFETY:`.
  - **158 blocs reçoivent la phrase de repli** « invariant non établi par lecture locale ; revue demandée (programme 260). » (appels à `from_raw_fd`, `set_var`, `assume_init`, `mkfifo` avec chemin C, API Mach, rappels SQLite...). Principaux fichiers : `daemon_presence_tests.rs` 24, `workspace_commit_view.rs` 19, `attach_tests.rs` 11, `host_resources.rs` 8, `daemon.rs` 8. Un relecteur doit les établir un à un : travail laissé ouvert.
  - L'outil ne vérifie pas la longueur des tampons au-delà de la forme `x.as_ptr(), x.len()`.
- Non vérifié : la justesse sémantique des 329 phrases précises au-delà de la forme syntaxique contrôlée (choix assumé : seules les formes vérifiables par lecture du bloc reçoivent une phrase précise).

### Bilan lot 6
Seul `effective_uid()` modifie du code exécutable (équivalent). Les commentaires ne changent pas le comportement. « Activation en attente ».

### Lot 5 - Aiguillage découpé (T054 à T068)
- Statut : fait
- Commits : préparation du module `daemon/wrapper_dispatch.rs`, 37 commits `refactor(daemon): Extraire la branche <Nom> de l'aiguillage`, extraction des contrôles `service_refusal` et `client_refusal`, nettoyage des imports et `mut` devenus inutiles.
- Fichiers : `crates/bridget-daemon/src/daemon.rs`, `crates/bridget-daemon/src/daemon/wrapper_dispatch.rs`, `outils/extraire_branches.py` (nouvel outil : lit la branche, lit les types dans l'énumération du protocole, écrit `on_<nom>` avec le corps inchangé), `validation/lot05-depart.txt`, `validation/lot05-fin.txt`.
- Vérifications : fonction `handle_wrapper_message_with_peer` : 5 010 lignes au départ -> 1 296 lignes (SC06 : moins de 1 500). 37 branches de 30 lignes ou plus extraites, plus aucune branche de 30 lignes ou plus dans le `match`. `clippy --all-targets -D warnings` : code 0. Porte sur cliché isolé de `HEAD` : `C-NOMS` identique à la base ; `C-ECHECS` = base + 1 test (`native149_o4_lot_mixte...`) qui passe trois fois en rejeu seul (noté instable).
- Écarts :
  - Les commits de branche sont automatiques (outil + compilation du démon avant chaque commit). Quinze branches ont d'abord échoué faute d'imports de types : l'outil qualifie maintenant les types par leur chemin complet et le module importe `bridget_transport::protocol::*`.
  - Le corps des branches est copié tel quel (y compris l'indentation d'origine, que le lot 10 normalisera).
  - Les deux blocs de refus ont été extraits en un seul commit.
- Non vérifié : rien de plus.

### Bilan lot 5
Code exécutable modifié (aiguillage du démon) : **activation en attente**.

### Lot 7 - Concurrence observable (T081 à T091)
- Statut : fait. T090 (regroupement des requêtes) différée : porte D7.
- Commits : `poison.rs` (trait `RecoverPoison`, compteur, 2 tests) ; relevé (`validation/lot07-sites-avant.txt`) ; remplacement de 853 sites dans 55 fichiers ; compteur de connexions, maximum et durée du tour (`feat(daemon): Compter les connexions actives, leur maximum et la duree du tour`).
- Vérifications : 853 sites `unwrap_or_else(|e| e.into_inner())` remplacés par `.recover_poison()` (0 restant) ; le diff ne contient que ce motif et les imports. Porte sur cliché isolé de `HEAD` : `C-ECHECS` identique à la base (24 échecs) ; `C-NOMS` = base + exactement 3 tests ajoutés (2 de `poison`, 1 de `Metrics`). La base de noms est actualisée.
- Écarts :
  - T088 et T089 sont dans un seul commit. Le compteur de connexions est décrémenté après le `catch_unwind` du fil de connexion (aucune sortie anticipée possible) : pas de structure de garde, donc aucune abstraction ajoutée.
  - Deux imports du trait sont conditionnés aux tests (`act_kind.rs`, `communication/client.rs`) parce que le code qui l'appelle n'existe qu'en test.
  - Aucune trace ni compteur n'est exposé dans `get_status` (pas de changement de protocole).
  - Seuil de lenteur du tour : 250 ms (provisoire, commenté dans le code).
- T090 : différée (porte D7 en attente). Après activation, relever le nombre de lignes de journal « tour de délégation lent » par heure et la durée maximale ; les requêtes à étudier sont dans `native_delegation.rs`, fonction `tick` (`open_requests()` puis `for_mission`, `failed_for_owner`). Aucun code de regroupement n'a été écrit.
- Non vérifié : les mesures réelles (activation en attente).

### Bilan lot 7
Code exécutable modifié (traces et compteurs) : **activation en attente**.

### Lot 8 - Erreurs sans texte fragile (T092 à T098)
- Statut : T092, T093 à T095 (partielles), T097, T098 faits ; T096 non faite.
- Commits : `refactor(daemon): Partager deux textes d'erreur entre emetteur et comparaisons`, correction du classement.
- Fichiers : `daemon.rs`, `daemon/wrapper_dispatch.rs`, `daemon/native_workspace_preparation210.rs`, `daemon/native_delegation.rs`, `validation/lot08-classement.md`.
- Vérifications : inventaire de 52 comparaisons de texte en production (et non 375 : le relevé initial comptait d'autres receveurs et des tests en ligne) ; trois constantes partagées (`CROSS_PROJECT_REASON_REQUIRED`, deux préfixes `REMOTE_*_PREPARE_PREFIX`) remplacent 16 littéraux identiques, texte inchangé au caractère près ; `clippy -D warnings` code 0. Porte combinée avec le lot 9 (voir plus bas).
- Écarts : 45 comparaisons restent non traitées (paires entre fichiers, textes composés, phrases de guidage) ; aucune annotation « texte externe » (T096) n'a été écrite faute d'avoir établi une source externe ; T097 fermée par D13 (aucun niveau 2, aucune liste candidate rédigée au-delà du classement).
- Non vérifié : rien de plus.

### Lot 9 - Fonctions lisibles (T099 à T113, partiel)
- Statut : T099, T100, T102, T103, T112, T113 faites ; T101, T104 arrêtées ; T105 à T111 non faites.
- Commits : cinq structures d'arguments (`prompt_request`, `resolve_target`, `reconnect_managed_session`, `connect_and_register_at`, `connect_and_register_with_domain_at`), exemptions et annotations.
- Fichiers : `outils/regrouper_arguments.py` (nouvel outil), `outils/longues_fonctions.py`, `validation/lot09-restes.md`, `lot09-restes-longues.txt`, `lot09-restes-params.txt`.
- Vérifications : chaque regroupement compile (`C-CHECK` complet avant chaque commit). Porte (lots 8 et 9 ensemble) sur cliché isolé de `HEAD` : `C-ECHECS` = base + le test instable connu `reprise_daemon_redelivre...` ; `C-NOMS` identique à la base (0 ligne de différence).
- Écarts :
  - **SC09 non atteint** : `launch_session_with_status` (1 359 lignes) et `spawn_reader` de `codex_app_server.rs` (1 098 lignes) restent au-dessus de 1 000 lignes. Le découpage manuel (T105 à T111) n'a pas été entrepris : variables locales partagées sur plusieurs centaines de lignes, risque élevé, sans outil de vérification autre que les tests.
  - Quatre fonctions à 15 paramètres ou plus restent sans structure (`handle_register_with_channel`, `post`, `apply`, `spawn_reader`) : voir `lot09-restes.md`.
  - Les lots 8 et 9 partagent une seule porte de tests (écart par rapport à une porte par lot, pour limiter la durée).
- Non vérifié : rien de plus.

### Bilan lots 8 et 9
Code exécutable modifié (constantes, signatures internes) : **activation en attente**.
