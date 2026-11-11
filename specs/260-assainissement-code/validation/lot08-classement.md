# Classement des comparaisons de texte d'erreur - lot 8

Relevé du 11 novembre 2026 (`rg` sur `crates` hors fichiers de tests nommés, motif du guide). 52 occurrences, pas 375 : les 375 du relevé initial comptaient d'autres receveurs et les tests en ligne.

| Classe | Définition | Comparaisons | Traitement |
|--------|------------|--------------|------------|
| A traitée | texte produit par Bridget, comparé ailleurs, littéral identique des deux côtés | 7 comparaisons (16 remplacements avec les producteurs) | constante partagée (voir ci-dessous) |
| Non traitées | 45 comparaisons : paires entre fichiers (ex. `workspace_path_unsafe`, `native_cancelled`), littéraux identiques sans lien (`"private"`, `"secret"`, `"terminal"`, `"65536"`), préfixes suivis d'une partie variable (`native_cancel_unverifiable:`, `remote_start_failed:remote_release:`), phrases françaises de guidage comparées dans des tests en ligne | 45 | laissées ouvertes ; la ventilation fine n'a pas été faite (inventaire brut : voir la commande en tête de ce fichier) |

## Constantes créées (texte inchangé, caractère pour caractère)

| Constante | Fichier | Remplacements |
|-----------|---------|---------------|
| `CROSS_PROJECT_REASON_REQUIRED` | `daemon.rs` (visible par `wrapper_dispatch.rs` et `native_delegation.rs` via `super`) | 7 dans `daemon.rs`, 2 dans `daemon/wrapper_dispatch.rs` |
| `REMOTE_WORKSPACE_PREPARE_PREFIX`, `REMOTE_COMMIT_VIEW_PREPARE_PREFIX` | `daemon/native_workspace_preparation210.rs` (`pub(super)`) | 5 dans ce fichier, 2 dans `native_delegation.rs` |

Les littéraux des tests (`*_tests.rs`) restent en dur : un test qui écrit le texte détecte un changement non voulu.

## Non traité, laissé ouvert

Les 45 comparaisons « non traitées » ci-dessus. Aucune annotation « texte externe » (T096) n'a été écrite : aucun des textes relevés n'a été établi comme venant d'un système externe sans lecture de son producteur.
