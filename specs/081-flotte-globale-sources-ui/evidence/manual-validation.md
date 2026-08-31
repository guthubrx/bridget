# Validation manuelle - SPEC-081

Date : 2026-08-31

## Parcours relu

Le parcours défini dans quickstart.md a été confronté au code :

- Toute la flotte, Sources et les projets viennent exclusivement de fleet_snapshot.
- Les chips sont des boutons, donc retirables au clavier.
- La création et l'import demandent une source quand le filtre ne désigne pas une seule source.
- panel_open ne peut viser qu'un source_id actif ou la découverte locale vérifiée.
- La conversation distante est chargée dans le panneau enfant compact.
- Les épingles, exclusions, tris et groupes repliés sont dans PreferencesStore local.

## Limite de validation native

Le worktree est sur un serveur Linux. Une vérification croisée Apple a été tentée après installation de la cible Rust 1.92 :

    cargo check --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml --target aarch64-apple-darwin

Elle s'arrête dans objc2-exception-helper car le compilateur Linux ne comprend pas les options Apple -arch et -mmacosx-version-min. Il ne s'agit pas d'un échec du code SPEC-081, mais il n'est pas possible de confirmer la WebView macOS ni le rendu visuel depuis cette machine.

Validation restante avant livraison : exécuter quickstart.md sur un poste macOS avec Bridget Desktop, deux sources SSH et un relais local disponible.

## État final du gate

La limite native est assumée et ne doit pas être masquée : aucun essai graphique macOS n'a été réalisé depuis le serveur Linux. Tous les contrôles déterministes disponibles ici sont consignés dans evidence/validation.md.
