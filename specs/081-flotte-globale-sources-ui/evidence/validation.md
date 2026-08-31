# Preuves de validation - SPEC-081

Date : 2026-08-31
Worktree : /home/moi/bridget-referent/.worktrees/session-081-flotte-globale-sources-ui

## T036 - Témoins ciblés

| Commande | Résultat exact |
|---|---|
| PATH=/home/moi/.cargo/bin:$PATH cargo test --quiet --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml | Succès - 53 tests exécutés, 0 échec, 0 ignoré. |
| node --test apps/bridget-desktop/ui/fleet-presentation.test.mjs | Succès - 7 tests, 0 échec, dont projection de 200 agents en 1,076 ms. |
| node --test crates/bridget-daemon/assets/ui/app.js | Succès - 112 tests, 0 échec. Le témoin SPEC-081 de coque compacte, conversation et onboarding est vert. |
| node --check apps/bridget-desktop/ui/fleet-app.js et node --check apps/bridget-desktop/ui/fleet-presentation.js | Succès - syntaxe JavaScript validée. |

Les témoins couvrent les homonymes par clé composée, l'absence de chemin ou jeton dans la projection publique, l'isolement d'une source illisible, la source locale conditionnelle, les filtres indépendants, les tris/groupes imbriqués et l'épinglage explicite.

## T037 - Gates de qualité

| Commande | Résultat exact |
|---|---|
| PATH=/home/moi/.cargo/bin:$PATH cargo fmt --check | Succès, code formaté. |
| PATH=/home/moi/.cargo/bin:$PATH cargo clippy --workspace --all-targets -- -D warnings | Succès, 0 warning admis. |
| PATH=/home/moi/.cargo/bin:$PATH cargo test --workspace --quiet | Succès, code de sortie 0. Toutes les suites exécutées sont vertes ; les tests ignorés sont explicitement marqués comme tels par le workspace. |
| git diff --check | Succès, aucun espace parasite ni marque de conflit. |

La suite workspace comprend notamment une série de 759 tests terminée sans échec. Aucun échec n'a été masqué ni relancé sélectivement.

## T038 - Validation native et limites

Le parcours de quickstart.md a été relu contre les appels Tauri et les deux WebViews. La validation visuelle native reste à faire sur un poste macOS : le worktree se trouve sur Linux et ne possède ni SDK ni compilateur Objective-C Apple. La compilation croisée a échoué dans objc2-exception-helper sur les options Apple -arch et -mmacosx-version-min=11.0, avant toute compilation du code de la SPEC.

Cette limite ne concerne pas la logique testée. Elle interdit simplement d'affirmer une validation de rendu WebView macOS avant la livraison.

## Rejeu après intégration à la tête main

La branche a été rebased sur origin/main à 426b964. Les conflits de préférences Desktop ont été résolus en conservant les réglages de sécurité de contenu et les préférences de flotte. Le point d'entrée Desktop minimal délègue désormais cette configuration au contrôleur fleet-app.js, qui est celui effectivement exécuté.

Le rejeu a donné : cargo fmt --check vert, cargo clippy --workspace --all-targets -- -D warnings vert, cargo test --workspace --quiet vert avec le code de sortie 0, et les tests Node verts. La suite workspace inclut 759 tests verts et 7 ignorés dans sa suite principale ; aucun échec n'est présent.
