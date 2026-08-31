# Preuves de validation - SPEC-080

Date : 2026-08-31

## Relais effectivement déployé

- Build : `cargo build --release -p bridget-daemon` réussi.
- Installation : `/home/moi/.local/bin/bridget` SHA-256 `1ebce05302baee7131369f87549f6951c09a49c4a2ba73ff25a8a30f8e32f12f`.
- Services : `bridget-daemon.service` et `bridget-ui.service` actifs depuis 07:03:08 UTC.

## Contrôles automatisés

- `node --check crates/bridget-daemon/assets/ui/app.js` : PASS.
- `node crates/bridget-daemon/assets/ui/app.js` : PASS, 95 tests, 0 échec.
- `cargo fmt --check` : PASS.
- `cargo test -p bridget-daemon spec_080 --lib` : PASS, 2 tests.
- `node --check apps/bridget-desktop/ui/app.js` : PASS.
- `cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml panels` : PASS, 1 test.
- `git diff --check` : PASS.

## Régression de finition couverte

Le test UI vérifie que le centre contient l'overlay, la roue de bas de barre, la grille `SettingsRow` portée, le sélecteur de police de 11 rem, le sélecteur de taille de 5,5 rem et la bordure fine de contrôle. Cette preuve ne remplace pas l'évaluation visuelle humaine.

## Vérification manuelle restante

1. Ouvrir Bridget sur un profil déjà relié.
2. Ouvrir la roue en bas de la barre gauche, puis Typographie.
3. Vérifier les titres, sous-titres, deux sélecteurs alignés à droite et les aperçus sous les lignes.
4. Changer puis restaurer une préférence locale, et vérifier qu'aucune mutation serveur n'est déclenchée.
5. Vérifier la page Serveur sans appliquer de réglage, puis Usage et facturation sans supposer un coût absent.
6. Reconstruire puis installer le paquet macOS avant de vérifier le raccourci de réglages par profil.

Aucune validation visuelle ambiguë, aucun clic non confirmé et aucune opération de serveur non demandée ne sont comptés comme preuve d'acceptation.
