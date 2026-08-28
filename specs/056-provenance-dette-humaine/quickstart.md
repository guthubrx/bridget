# Rejeu ciblé — Session 056

Depuis la racine du worktree, avec un `CARGO_TARGET_DIR` dédié :

1. lister les tests de provenance avant exécution ;
2. jouer les tests de domaine et de store de la tranche 1 ;
3. jouer les tests `delegate`, routines et guichet ;
4. exécuter `cargo check --workspace --all-targets` ;
5. exécuter `cargo fmt --all -- --check` en édition workspace 2024 ;
6. vérifier le diff et l'absence de trace temporaire.

Les commandes et lignes natives exactes sont consignées dans
`implementation.md` au moment de la livraison.
