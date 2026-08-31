# Implémentation - SPEC-080 Centre de contrôle Bridget

Date : 2026-08-31
Statut : tranche de centre de contrôle déployée sur le relais et paquet macOS construit. La SPEC reste ouverte tant que la validation visuelle humaine et le remplacement explicite de l'application macOS en cours ne sont pas réalisés.

## Résultat présent dans le relais déployé

- Une roue seule, accessible et ancrée sous la liste des agents, ouvre un `dialog` modal. La conversation reste visible derrière l'overlay.
- L'overlay comprend Général, Apparence, Date et heure, Typographie, Serveur, Usage et facturation, Mises à jour et Diagnostics, avec recherche locale.
- Nom affiché, thème, fuseau IANA, police d'interface, police monospace, tailles et retour à la ligne restent dans le stockage local du WebView. Ces préférences n'appellent aucune route de réglage serveur.
- La page Serveur ne propose que les capacités publiées. La politique des racines de projets est prévisualisée puis confirmée avant application. Les autres catégories restent explicitement en lecture seule.
- Usage et facturation affiche les jetons attestés par fournisseur et modèle. En l'absence d'une grille tarifaire datée, elle indique qu'aucune estimation API ne peut être produite.
- Mises à jour et Diagnostics restent informatifs : aucune action d'hôte, de fournisseur ou de maintenance n'est envoyée par cette surface.

## Portage de finition T3 Code

La finition de l'overlay conserve les fonds sombres et les variables de couleurs Bridget. Le gabarit des réglages est en revanche porté du composant de réglages T3 Code installé :

- grille à deux colonnes `SettingsRow`, libellé et sous-titre à gauche, contrôle à droite ;
- tailles, densité, hiérarchie typographique, focus et sélecteurs de type `SelectTrigger` repris dans la feuille de style locale ;
- aperçus sous leur ligne de réglage et non dans une carte générique ;
- bordure d'overlay unique, fine, et coins très légèrement arrondis, conformément au choix Bridget.

Les contrôles restent des éléments HTML natifs et accessibles. Seule leur présentation a été adaptée, aucune couleur de fond Bridget n'a été remplacée par le thème T3.

## Déploiement attesté

Le binaire release courant a été construit depuis ce worktree, installé dans /home/moi/.local/bin/bridget, puis les services utilisateur ont été redémarrés le 2026-08-31 à 07:12:31 UTC :

- `bridget-daemon.service` : actif, PID 2280068 ;
- `bridget-ui.service` : actif, PID 2280069 ;
- SHA-256 du binaire installé : `1ea5b75d3b4f5f670c0b939ea09e5ce45728ca434ee4e5ddab1525ed87cf99f9`.

## Preuves automatisées

| Commande | Résultat |
|---|---|
| `node --check crates/bridget-daemon/assets/ui/app.js` | PASS |
| `node crates/bridget-daemon/assets/ui/app.js` | PASS - 97 tests, 0 échec |
| `cargo fmt --check` | PASS |
| `cargo test -p bridget-daemon spec_080 --lib` | PASS - 2 tests ciblés |
| `cargo build --release -p bridget-daemon` | PASS |
| `node --check apps/bridget-desktop/ui/app.js` | PASS |
| `cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml panels` | PASS - 1 test ciblé |
| `git diff --check` | PASS |
| `cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml --test desktop_commands` | PASS - 2 tests ciblés |


## Correctif des actions de cartes Desktop

- Une régression du montage de l'interface Desktop avait supprimé la délégation des actions de cartes. Les boutons Connecter, Réessayer, Ouvrir le relais, Déconnecter, Modifier, Retirer et Réglages sont à nouveau routés par le listing.
- Le test `desktop_commands::les_actions_de_carte_sont_delegatees_au_listing_de_profils` atteste les six branches d'action et échoue si la délégation ou les appels Connecter et Réglages disparaissent.
- La sonde SSH avec les arguments exacts de Bridget atteste que `moi@cartae.app:2222` est joignable. Le profil local Carte contient encore `Moi` et devra être corrigé en `moi` après fermeture de Bridget, afin de ne pas faire réécrire l'état en mémoire.

## Correctif de panneau unique et d'icône macOS

- La fenêtre affichait deux copies du panneau lorsque deux profils étaient ouverts : le registre autorisait deux WebViews enfants et `arrange_panels` partageait la largeur de la fenêtre entre elles. Les deux serveurs aboutissaient au même relais, d'où deux interfaces visuellement identiques.
- Bridget conserve désormais un seul panneau distant. L'ouverture d'un serveur ferme le panneau précédent et affiche le nouveau sur toute la fenêtre, sans fermer les tunnels déjà établis.
- Le registre est limité à un panneau et les tests couvrent la limite, la fermeture du panneau précédent et le rendu pleine largeur.
- `bridget.svg` est converti en `apps/bridget-desktop/src-tauri/icons/icon.icns`, déclaré dans `tauri.conf.json` et contrôlé dans le bundle macOS. Le paquet contient désormais `Contents/Resources/icon.icns` et `CFBundleIconFile=icon.icns`.

## Paquet macOS construit

- Bundle : `Bridget.app`, reconstruit avec `cargo tauri build --bundles app` après le correctif de panneau unique et l'intégration de l'icône.
- Manifeste : nom affiché et nom de bundle `Bridget`, identifiant `app.cartae.bridget-desktop`, version `0.1.0`.
- Icône : `CFBundleIconFile=icon.icns` et `Contents/Resources/icon.icns` sont présents dans le bundle.
- Signature : signature ad hoc vérifiée par `codesign --verify --deep --strict`. Le paquet n'est pas notarisé Apple, ce qui est attendu pour cette distribution interne.

## Limites explicites avant clôture

- Le paquet corrigé doit être installé dans `/Applications/Bridget.app` seulement après fermeture explicite de l'instance en cours, puis validé visuellement avec Loin et Carte.
- La validation humaine de la vue Typographie après le portage T3 reste requise. Aucun résultat de clic ou de capture externe ambiguë n'est compté comme acceptation.
- Les coûts restent indisponibles sans tarifs versionnés. Les filtres avancés, la courbe quotidienne et les autres réglages serveur écrits restent des tâches non cochées dans `tasks.md`.
