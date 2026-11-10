# Lot 1 - Filet automatique (T008-T024)

But : une commande locale et un contrôle distant qui refusent tout nouvel avertissement. Mesures : `releves.md` sections 2 et 15.
Branche : `session-260-lot01-filet`. Worktree : `.worktrees/260-lot01-filet`. Règles : `contracts/regles-implementeur.md`. Contrôles distants : `contracts/controles-ci.md`.
Premier commit avant le premier Cargo (R1). Code exécutable modifié à partir de T011 : « activation en attente » dans le journal (R8).

## T008 - Cibles du Makefile

Fichier : `Makefile`. Chercher la cible `test:` et copier son style (tabulations).
Ajouter quatre cibles, avec `.PHONY` :

| Cible | Commande |
|-------|----------|
| `fmt` | `cargo fmt --all` |
| `fmt-check` | `cargo fmt --all -- --check` |
| `clippy` | `python3 scripts/build.py cargo clippy --workspace --all-targets --locked --features bridget-daemon/test-support` |
| `lint` | dépend de `clippy` et `fmt-check` |

Vérifier : `make -n lint` affiche les commandes sans les lancer. Résultat attendu : deux commandes. Ne pas lancer `make fmt`. Commit : `chore(build): Ajouter les cibles fmt, fmt-check, clippy et lint`.

## T009 - Lints du workspace

Fichiers : `Cargo.toml` (racine), `crates/*/Cargo.toml` (3 fichiers).
1. Racine : ajouter `[workspace.lints.clippy]` avec ces `allow`, chacun avec un commentaire `# dette structurelle, traitée au lot 9 du programme 260` :
   `too_many_arguments = "allow"`, `type_complexity = "allow"`, `result_large_err = "allow"`, `large_enum_variant = "allow"`.
2. Dans chacun des 3 `Cargo.toml` de crate : ajouter `[lints]` puis `workspace = true`.
3. Vérifier : `C-CHECK`. Attendu : code 0. Puis `C-CLIPPY` : le nombre d'avertissements baisse d'environ 78 par rapport à `validation/base-clippy.txt`.
Ne pas ajouter d'autre lint ici. Commit : `chore(lint): Declarer les lints du workspace`.

## T010 - Workflow `rust.yml`

Créer `.github/workflows/rust.yml` selon `contracts/controles-ci.md`, fichier 1. Les trois jobs sont en `continue-on-error: true`.
Vérifier la syntaxe : `python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/rust.yml'))"` (si `yaml` manque, `ruby -ryaml -e 'YAML.load_file(".github/workflows/rust.yml")'`).
Puis pousser la branche de lot et lire un passage réel : `gh run list --workflow rust.yml --limit 1`. Attendu : le fichier est accepté (un état « completed » quel que soit le résultat). Fichier refusé : corriger la syntaxe. Noter « non exécuté » si `gh` n'est pas utilisable. Commit : `ci(rust): Ajouter le contrôle Rust en observation`.

## T011 - `never_loop` (D2) - première modification de code exécutable

Fichier : `crates/bridget-daemon/src/daemon/native_lineage.rs`. Chercher : `while receiver.recv().is_ok() {`.
Remplacer `while` par `if`. Ne rien changer d'autre. La boucle n'itère jamais plus d'une fois : comportement constant.
Vérifier : `C-CHECK`, puis `C-CRATE` avec `<crate>` = `bridget-daemon`. Attendu : mêmes échecs que `base-echecs.txt` pour ce crate.
Si D2 est « en attente » : appliquer ce défaut quand même et l'écrire dans le journal. Commit : `fix(daemon): Remplacer la boucle qui ne boucle jamais par une condition`.

## T012 - `non_octal_unix_permissions`

Fichier : `crates/bridget-daemon/src/daemon/native_workspace154_tests.rs`. Chercher : `from_mode(0)`. Remplacer par `from_mode(0o000)`. Vérifier : `C-CHECK`. Commit : `style(tests): Ecrire le mode 000 en octal`.

## T013 - `reversed_empty_ranges`

Fichier : `crates/bridget-daemon/src/daemon/mobile_operations.rs`. Chercher : `1=>1..=0`.
La plage vide est voulue. Ne pas appliquer la suggestion. Ajouter au-dessus de l'élément le plus proche : `#[allow(clippy::reversed_empty_ranges)] // plage vide voulue : aucun cas pour ce rang`. Le texte du commentaire doit rester vrai : lire le contexte (10 lignes autour) et ajuster la phrase.
Vérifier : `C-CHECK`. Commit : `style(daemon): Documenter la plage vide voulue`.

## T014 - `collapsible_if` (110 cas)

Commande (une règle, tout le workspace) :
`python3 scripts/build.py cargo clippy --fix --allow-dirty --allow-staged --workspace --all-targets --locked --features bridget-daemon/test-support -- -A clippy::all -W clippy::collapsible_if`
Vérifier : `git diff --stat` (beaucoup de fichiers, petites modifications), `C-CHECK`, `C-CRATE` pour les 2 crates les plus touchés. Si la correction automatique échoue sur un cas : le laisser, noter son lieu. Pas de `cargo fmt` (règle Format). Commit : `refactor(lint): Fusionner les conditions imbriquees`.

## T015 - Autres règles à correction automatique

Même commande que T014, **une règle par commit**, dans cet ordre :
`needless_borrows_for_generic_args`, `needless_borrow`, `useless_conversion`, `redundant_field_names`, `cloned_ref_to_slice_refs`, `let_and_return`, `question_mark`, `unnecessary_get_then_check`, `identity_op`.
Après chaque règle : `git diff --stat`, `C-CHECK`. `C-CRATE` toutes les 5 règles. Commit : `refactor(lint): Appliquer la règle <nom>`. Une règle sans correction automatique : passer à T017.

## T016 - `nonminimal_bool` et `bool_assert_comparison`

Même commande, un commit par règle. Relire le diff de `nonminimal_bool` : la logique booléenne doit rester équivalente. Un doute sur un cas : l'annuler (`git checkout -- <fichier>`) et noter son lieu. `bool_assert_comparison` : `assert_eq!(x, true)` devient `assert!(x)`.

## T017 - Petites règles restantes

Lancer `C-CLIPPY` avec `<nom>` = `lot01-t017`. Lire les avertissements restants (`rg '^warning' $V/lot01-t017-clippy.txt | sort | uniq -c | sort -nr`). Pour chaque règle restante hors T018 : correction automatique si possible, sinon correction à la main du cas (moins de 5 lignes chacun). Cas qui change le sens du code : `#[allow(...)]` local avec commentaire qui dit pourquoi. Un commit par règle ou par fichier.

## T018 - `zombie_processes` (5 cas)

Chercher : `rg 'Command::new' crates --glob '*.rs'` dans les lieux signalés par clippy. Un processus lancé et jamais attendu devient un zombie. Ajouter l'attente (`.wait()`) seulement si le code du lieu le permet sans changer le comportement ; sinon `#[allow(clippy::zombie_processes)]` avec le commentaire qui dit qui attend le processus. Si le doute persiste : ne pas modifier, noter (R4).

## T019 - Analyse sans avertissement, puis bloquante

1. `C-CLIPPY` avec `-- -D warnings` : ajouter `-- -D warnings` à la fin de la commande. Attendu : code 0.
2. Si des avertissements restent : retourner à T017. Au plus 2 allers-retours, puis noter les restes et arrêter la tâche.
3. Dans `.github/workflows/rust.yml` : retirer `continue-on-error: true` du job `clippy`.
4. Dans le `Makefile`, la cible `clippy` ajoute `-- -D warnings`.
Commit : `ci(rust): Rendre l'analyse statique bloquante`.

## T020 - Workflow `rust-audit.yml`

Créer `.github/workflows/rust-audit.yml` selon `contracts/controles-ci.md`, fichier 2. Valider comme en T010. Écrire la version de `cargo-audit` choisie dans le journal. Commit : `ci(audit): Ajouter l'audit hebdomadaire des dépendances`.

## T021 - Ligne de base Linux

Après poussée de la branche, lancer le job `tests` trois fois (`gh workflow run rust.yml --ref session-260-lot01-filet` puis `gh run list`). Pour chaque passage : `gh run view <id> --log | grep -E '^test .* \.\.\. FAILED' | sort`.
Écrire `validation/base-linux-echecs.txt` = lignes présentes dans les 3 passages. Écrire dans le journal les lignes présentes dans un ou deux passages seulement (tests instables).
Si le job ne démarre pas (workflows désactivés sur le dépôt) : noter « non exécuté », la ligne de base Linux reste inconnue, passer à T022.

## T022 - Documentation

Ajouter à la documentation de développement la section « Contrôles avant livraison » : `make lint`, `make test`, la règle « un avertissement refuse la livraison », le renvoi vers `contracts/controles-ci.md`. Fichier : le premier existant parmi `docs/DEVELOPMENT.md`, `README.md` (choisir par `ls docs`). Garder la section sous 15 lignes. Lancer `python3 scripts/check-docs.py` : attendu code 0.

## T023 - Porte du lot

`C-TESTS` (`<nom>` = `lot01`), puis `C-ECHECS`. Attendu : aucune ligne de différence. `C-CLIPPY -- -D warnings` : code 0. Écrire le bilan dans `implementation.md`. Fin de lot (`quickstart.md`).

## T024 - Tests bloquants (D16)

Lire D16 dans `decisions.md` (`origin/main`). Statut `tranchée` « oui » et `base-linux-echecs.txt` existe : retirer `continue-on-error` du job `tests` et exclure du contrôle les tests de `base-linux-echecs.txt` (option `--skip <nom>` pour chacun). Sinon : ne rien changer, écrire « différé (porte D16) ». Cocher la case dans les deux cas.

## Arrêts propres à ce lot

- Plus de 2 allers-retours sur une même règle après correction automatique : arrêter cette règle, la noter, passer à la suivante.
- `clippy --fix` modifie un fichier hors des lieux attendus (production sans lien avec la règle) : relire le diff ; en cas de doute, annuler ce fichier.
- Tout changement de comportement visible par un test : régression, `git revert` du commit fautif (R3).
