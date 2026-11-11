# Classement des comparaisons de texte d'erreur - lot 8

Relevé du 11 novembre 2026 (`rg` sur `crates` hors fichiers de tests nommés, motif du guide). 52 occurrences, pas 375 : les 375 du relevé initial comptaient d'autres receveurs et les tests en ligne.

| Classe | Définition | Nombre | Traitement |
|--------|------------|--------|------------|
| A traitée | texte produit par Bridget, comparé ailleurs, littéral identique des deux côtés | 11 sites (2 familles) | constante partagée (voir ci-dessous) |
| A paire inter-fichiers | producteur dans un autre fichier (ex. `workspace_path_unsafe` : `workspace_git.rs` -> `native_delegation.rs`) | 4 | non traitée (constante à rendre visible entre modules) |
| A coïncidence | même littéral ailleurs sans lien (ex. `"private"`, `"secret"`, `"terminal"`, `"65536"`) | 8 | aucune action |
| B texte composé | préfixe suivi d'une partie variable (`native_cancel_unverifiable:`, `remote_start_failed:remote_release:`, `cross_project_reason_required:`...) | 8 | non traitée |
| B texte français affiché | phrases de guidage (`autres travaux autorisés`, `non attestée`, `échéance`...) comparées dans les tests en ligne ou les projections | 17 | non traitée |
| C / D | doute ou comparaison locale | 4 | aucune action |

## Constantes créées (texte inchangé, caractère pour caractère)

| Constante | Fichier | Remplacements |
|-----------|---------|---------------|
| `CROSS_PROJECT_REASON_REQUIRED` | `daemon.rs` (visible par `wrapper_dispatch.rs` et `native_delegation.rs` via `super`) | 7 dans `daemon.rs`, 2 dans `daemon/wrapper_dispatch.rs` |
| `REMOTE_WORKSPACE_PREPARE_PREFIX`, `REMOTE_COMMIT_VIEW_PREPARE_PREFIX` | `daemon/native_workspace_preparation210.rs` (`pub(super)`) | 5 dans ce fichier, 2 dans `native_delegation.rs` |

Les littéraux des tests (`*_tests.rs`) restent en dur : un test qui écrit le texte détecte un changement non voulu.

## Non traité, laissé ouvert

Voir les classes « A paire inter-fichiers » et « B » ci-dessus. Aucune annotation « texte externe » (T096) n'a été écrite : aucun des textes relevés n'a été établi comme venant d'un système externe sans lecture de son producteur.
