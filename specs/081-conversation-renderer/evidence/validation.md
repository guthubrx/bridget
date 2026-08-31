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
| `node crates/bridget-daemon/assets/ui/app.js` après reprise US1 | succès, 112 tests | composition demande, activité, travail et réponse |
| `/Users/moi/.cargo/bin/cargo fmt --check` après reprise US1 | succès | format Rust |
| `env TMPDIR=/tmp /Users/moi/.cargo/bin/cargo test -p bridget-daemon ui` | 149 succès, 2 échecs de harnais | les assets UI compilent ; limites temporaires documentées ci-dessous |
| `env TMPDIR=/tmp /Users/moi/.cargo/bin/cargo test -p bridget-daemon ui::tests::assets_statiques_annoncent_etag_et_revalidation` | succès, 1 test | livraison et revalidation des assets servis par le daemon |
| `/Users/moi/.cargo/bin/cargo build --release -p bridget-daemon` | succès | binaire release de la reprise US1 |

## Suite Rust UI complète

Avec le répertoire temporaire système usuel, la suite ouvre des sockets Unix
dont le chemin dépasse `SUN_LEN`. Avec `TMPDIR=/tmp`, elle atteint 149 succès,
mais deux limites de harnais restent observées : une assertion SPEC-080 compare
la forme logique `/tmp` à la forme canonique `/private/tmp`, et un test de
présence reçoit `Operation not permitted` en écrivant son état temporaire.
Ces deux échecs ne proviennent pas des assets JavaScript ou CSS de la reprise
US1 et ne sont pas présentés comme verts.

## Binaire de validation et déploiement

Le binaire release de la reprise US1 a été construit dans le worktree isolé,
puis installé après sauvegarde du binaire précédent. Son SHA-256 est
`4322e26aae5ca17d219dfb0044aa83ef427380f06da0f67f613d1833c2f1ecb3`.
Il est installé dans `/Users/moi/.local/bin/bridget`. La sauvegarde récupérable
est `/Users/moi/.local/bin/bridget.before-conversation-composition-20260831-144000`.

Le LaunchAgent `com.bridget.daemon` a été relancé. `bridget status` observe le
daemon en ligne, avec le build-id `0abc763db94e`, qui correspond à la tête
déployée de `main`. La validation visuelle manuelle T042 reste le verrou de
fermeture de la SPEC.

## Validation manuelle restante

Le scénario intégral de `quickstart.md` sur une fenêtre Bridget Desktop et un
serveur approuvé n'a pas été joué dans cette session. En particulier, le rendu
clair/sombre réel, le clic de navigateur système et l'aperçu de fichier via
tunnel restent à vérifier avant de déclarer la SPEC entièrement validée.
