# Vérification de la session 045

## Réalisation

- `send` et `reply` refusent avant connexion tout jeton d'option inconnu et le
  nomment dans le diagnostic.
- `--` termine explicitement l'analyse ; le reste devient le corps littéral.
- `--to` et `--from` utilisent la même validation de valeur manquante que les
  autres options textuelles.
- L'aide décrit la frontière `--` sans introduire `--body` ni `--no-reply`.
- La cohérence entre options reconnues reste hors périmètre.

## Oracles et cycle rouge-vert

Le harnais lance le vrai binaire contre une socket Unix jetable et capture le
JSON réellement transmis.

- base gelée : univers de 9 tests, `9 passed / 0 failed` ;
- tests ajoutés avant production : univers de 13 tests,
  `9 passed / 4 failed` ;
- tête restaurée : univers de 13 tests, `13 passed / 0 failed` ;
- unités `cli::hook_tests` : univers de 35 tests,
  `35 passed / 0 failed`.

Les quatre rouges initiaux étaient exactement : refus `send`, refus `reply`,
corps littéral après `--`, et valeur manquante de `--to`/`--from`.

## Mutants

1. Supprimer uniquement les gardes d'option inconnue dans les deux parseurs
   donne un univers exact de 2 tests puis `0 passed / 2 failed`. Les sorties
   montrent un code 0 et un message parti : les deux oracles mordent le chemin
   de production.
2. Supprimer uniquement les deux branches du séparateur donne un univers exact
   de 1 test puis `0 passed / 1 failed` ; le binaire refuse alors `--` au lieu
   de sérialiser le corps littéral.

Après restauration du mutant final, les condensats nominaux sont :

- `24a91c7cb84c830cbfb23b652376db5b3643276cec53a273d26f5ec20b87f9ab`
  pour `crates/bridget-daemon/src/cli.rs` ;
- `1f2d1c60dea222f7dcc673a0d115c36edd998bba8c50f06f6e3860d7578f161e`
  pour `crates/bridget-daemon/tests/cli_arguments_integration_test.rs`.

## Portes et limites

- `cargo check --workspace --all-targets` : vert.
- `rustfmt --edition 2024 --check` sur les deux fichiers Rust : vert.
- `git diff --check` : vert.
- Clippy strict `--all-targets --no-deps -D warnings` : rouge sur la base et la
  tête avec les mêmes 10 diagnostics dans `wrapper.rs`, `daemon.rs`,
  `managed_supervisor.rs`, `ui.rs` et `store.rs`. Aucun diagnostic ne vise les
  fichiers du lot.
- La campagne daemon complète n'a pas été exécutée : le périmètre mesuré est le
  harnais d'arguments, les unités CLI et la compilation de toutes les cibles.

## REX — Retour d'expérience

**Date** : 2026-08-27
**Tâches complétées** : 6/7 avant composition et publication finales

### Ce qui a fonctionné

- Capturer le message sérialisé a rendu visible la corruption que le code 0
  masquait.
- Le séparateur explicite couvre à la fois le refus strict et l'intention
  légitime d'envoyer un tiret.
- Les spécimens du ledger ont fourni des cas réels plutôt que des options
  inventées pour le test.

### Difficulté rencontrée

Un premier tir Clippy de référence est parti depuis le clone parent au lieu du
worktree de base nouvellement créé. Son résultat a été rejeté, puis la commande
a été rejouée depuis la tête `bc745335` attestée avant toute comparaison.

### Responsabilité future

Le changement ajoute deux branches identiques et une convention documentée,
sans dépendance ni abstraction. Il réduit la charge cognitive : une faute
d'option est désormais un refus local explicite, et le texte ambigu dispose
d'une frontière standard.
