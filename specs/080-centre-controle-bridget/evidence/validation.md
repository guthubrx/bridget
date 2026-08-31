# Preuves de validation - SPEC-080

Date : 2026-08-31

## Relais effectivement déployé

- Build : `cargo build --release -p bridget-daemon` réussi.
- Installation : `/home/moi/.local/bin/bridget` SHA-256 `1ea5b75d3b4f5f670c0b939ea09e5ce45728ca434ee4e5ddab1525ed87cf99f9`.
- Services utilisateur : `bridget-daemon.service` et `bridget-ui.service` actifs depuis 07:12:31 UTC.

## Contrôles automatisés

- `node --check crates/bridget-daemon/assets/ui/app.js` : PASS.
- `node crates/bridget-daemon/assets/ui/app.js` : PASS, 97 tests, 0 échec.
- `cargo fmt --check` : PASS.
- `cargo test -p bridget-daemon spec_080 --lib` : PASS, 2 tests.
- `node --check apps/bridget-desktop/ui/app.js` : PASS.
- `cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml panels` : PASS, 1 test.
- `git diff --check` : PASS.
- `cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml --test desktop_commands` : PASS, 2 tests.

## Régression de finition couverte

Le test UI vérifie que le centre contient l'overlay, la roue de bas de barre, la grille `SettingsRow` portée, le sélecteur de police de 11 rem, le sélecteur de taille de 5,5 rem et la bordure fine de contrôle. Cette preuve ne remplace pas l'évaluation visuelle humaine.

Le test UI couvre aussi la séparation de portées dans la navigation Projets : l'ancien pied global `Réglages / Retirer` est absent, l'overlay de présentation du projet est présent, les préférences d'initiales et de couleurs sont tolérantes aux valeurs invalides et restent locales, la colonne repliée conserve la liste d'icônes, et `Commande + virgule` ouvre le même centre de contrôle.

## Correctif des actions Desktop

- La délégation de clic du listing de profils couvre Connecter, Réessayer, Ouvrir, Déconnecter, Modifier et Retirer.
- La carte Desktop ne rend plus de lien textuel Réglages. La roue de la barre basse du panneau distant est l'unique entrée de réglages serveur.
- Le test Desktop statique atteste les cinq branches et l'absence de `data-action="settings"` ainsi que de `openServerSettings`.
- Sonde SSH des arguments de production : `moi@cartae.app:2222` réussit. Le profil Carte enregistré en `Moi` est refusé par SSH et doit devenir `moi` après fermeture de l'application.

## Correctif de duplication des panneaux et de l'icône

- Cause confirmée : deux WebViews enfants étaient autorisées et redimensionnées à `largeur / nombre de panneaux`. Avec Loin et Carte ouverts, la fenêtre affichait donc deux interfaces reliées côte à côte.
- Le registre Desktop est limité à un panneau. Toute ouverture ferme le panneau visible avant de créer le suivant, sans fermer le tunnel SSH associé.
- `cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml panels` : PASS, 1 test.
- `cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml --test two_panels` : PASS, 1 test.
- `cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml --test desktop_commands` : PASS, 3 tests.
- Le bundle macOS reconstruit contient `CFBundleIconFile=icon.icns` et `Contents/Resources/icon.icns`; sa signature ad hoc passe `codesign --verify --deep --strict`.

## Finition de la liste d'agents

- La grille d'agents est ancrée en haut avec `align-content: start` et des lignes `min-content`, afin qu'un petit nombre d'agents ne soit jamais réparti sur toute la hauteur de la barre.
- La recherche a une marge basse de `1rem` avant la liste.

## Paquet macOS attesté

- Bundle reconstruit avec un seul panneau distant et l'icône Bridget intégrée.
- Manifeste vérifié : `CFBundleDisplayName=Bridget`, `CFBundleName=Bridget`, `CFBundleIdentifier=app.cartae.bridget-desktop`.
- Intégrité locale : `codesign --verify --deep --strict` passe après signature ad hoc.

## Rectification de l'icône validée

- L'icône restaurée provient de l'ancien build qui a servi de référence visuelle, non du fichier `/Users/moi/Downloads/bridget.svg` qui avait perdu le fond noir et l'échelle de mascotte.
- Les fichiers source et macOS ont le même rendu attendu : fond noir légèrement dégradé, mascotte rose volontairement grande.

## Vérification manuelle restante

1. Ouvrir Bridget sur un profil déjà relié.
2. Ouvrir la roue en bas de la barre gauche, puis Typographie.
3. Vérifier les titres, sous-titres, deux sélecteurs alignés à droite et les aperçus sous les lignes.
4. Changer puis restaurer une préférence locale, et vérifier qu'aucune mutation serveur n'est déclenchée.
5. Vérifier la page Serveur sans appliquer de réglage, puis Usage et facturation sans supposer un coût absent.
6. Fermer Bridget, vérifier que Carte utilise `moi`, installer le bundle corrigé dans `/Applications/Bridget.app`, reconnecter Loin puis Carte et vérifier qu'un seul panneau pleine largeur reste visible et que l'icône Bridget est présente dans le Dock.
7. Vérifier sous les boutons macOS la colonne Projets : « Toute la flotte » doit expliciter « Tous projets confondus », un projet doit afficher une tuile avec initiales, et le pied de colonne ne doit afficher ni « Retirer » ni « Réglages ».
8. Ouvrir `…` ou faire un clic droit sur un projet, changer ses initiales puis sa couleur dans l'overlay, refermer puis vérifier la persistance locale. Replier la colonne et vérifier que les tuiles restent visibles.
9. Presser `Commande + virgule`, vérifier que le même overlay de centre de contrôle s'ouvre au-dessus de la conversation.

Aucune validation visuelle ambiguë, aucun clic non confirmé et aucune opération de serveur non demandée ne sont comptés comme preuve d'acceptation.
