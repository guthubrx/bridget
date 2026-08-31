# Vérification manuelle - SPEC-081

## Préconditions

- Bridget Desktop lancé.
- Deux profils SSH approuvés, connectés.
- Deux sources compatibles, avec si possible deux agents homonymes.
- Bridget local lancé, pour vérifier l'apparition de « Cet ordinateur » sans aucune configuration de jeton.

## Parcours

1. « Toute la flotte » affiche les agents de chaque source avec source et projet.
2. Cliquer une source crée un chip. Cliquer son projet crée un chip indépendant.
3. Retirer l'un des deux chips sans retirer l'autre.
4. Ajouter Source, État, Projet au tri, les inverser puis les réordonner.
5. Ouvrir deux homonymes : chaque conversation doit provenir du bon serveur.
6. Ouvrir le gestionnaire de serveurs pendant une conversation.
7. Couper un tunnel : seule sa source devient indisponible.
8. Épingler puis désépingler, relancer Desktop et vérifier la persistance locale.
9. Vérifier le pré-épinglage d'un coordinateur explicitement marqué et l'absence d'heuristique pour les anciens agents.
10. Vérifier l'absence de chemin projet et de détails SSH dans le DOM ou les réponses Desktop.
11. Arrêter Bridget local puis rafraîchir : « Cet ordinateur » disparaît sans dégrader les sources SSH.

## Preuves

- Tests Rust et JavaScript ciblés verts.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` et `cargo test --workspace`.
- Audit de réutilisation, Analyze, convergence et contre-revue adverse.
