# Preuves de validation - SPEC-080

Date: 2026-08-31

## Validation automatisée disponible

- `node crates/bridget-daemon/assets/ui/app.js`: 94 tests passants, 0 échec.
- `node --check apps/bridget-desktop/ui/app.js`: syntaxe valide.
- `git diff --check`: aucune erreur d'espacement détectée sur les fichiers suivis.

## Validation indisponible

- `cargo`, `rustc` et `rustfmt` sont absents de la machine de travail et du serveur K3s. Les tests Rust, le formatage Rust et la construction macOS n'ont donc pas été exécutés.
- Aucun serveur enregistré n'a été ouvert pour une vérification manuelle. Aucun réglage n'a été appliqué à un serveur réel.

## Décision de livraison

Ne pas déployer cette tranche tant que la chaîne Rust et une validation manuelle du parcours aperçu, confirmation, conflit et rejeu ne sont pas disponibles.
