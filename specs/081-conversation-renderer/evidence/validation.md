# Validation - SPEC-081

Le point de départ est consigné dans `implementation.md`. Les commandes ci-
dessous ont été rejouées après l'implémentation le 2026-08-31.

| Commande | Résultat observé | Portée |
|---|---|---|
| `node crates/bridget-daemon/assets/ui/app.js` | succès, 110 tests | tours, copie, Markdown hostile, préférences, références, ancre de lecture |
| `/Users/moi/.cargo/bin/cargo fmt --check` | succès | format Rust |
| `/Users/moi/.cargo/bin/cargo test -p bridget-daemon spec_081_apercu_fichier_reste_borne_canonique_et_sans_chemin_racine` | succès, 1 test | canonicalisation, racine, taille, chemin relatif et base64 |
| `/Users/moi/.cargo/bin/cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml preferences_store --lib` | succès, 1 test | V2, défaut sûr, corruption, V1 opérateur et permissions |
| `/Users/moi/.cargo/bin/cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml --test desktop_commands --test secrets_and_diagnostics` | succès, 7 tests | snapshot, rechargement natif, navigation externe, absence de capability, secrets |
| `shasum -a 256 -c SHA256SUMS` depuis `crates/bridget-daemon/assets/ui/vendor` | succès, 5 empreintes | vendor Markdown et coloration |
| `git diff --check` | succès | espaces et patch |

## Suite Rust UI complète

`/Users/moi/.cargo/bin/cargo test -p bridget-daemon ui` termine avec 137
succès et 10 échecs. Les dix échecs sont tous antérieurs à SPEC-081 et ont la
même cause observée : `path must be shorter than SUN_LEN` avec le worktree
temporaire `/private/tmp/bridget-project-nav.JNWHqE`. Le test SPEC-081 ciblé
ci-dessus passe dans ce même worktree. Cette suite n'est donc pas présentée
comme verte.

## Validation manuelle restante

Le scénario intégral de `quickstart.md` sur une fenêtre Bridget Desktop et un
serveur approuvé n'a pas été joué dans cette session. En particulier, le rendu
clair/sombre réel, le clic de navigateur système et l'aperçu de fichier via
tunnel restent à vérifier avant de déclarer la SPEC entièrement validée.
