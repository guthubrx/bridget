# Implémentation - SPEC-080 Centre de contrôle Bridget

Date : 2026-08-31
Statut : tranche de centre de contrôle déployée sur le relais. La SPEC reste ouverte tant que la validation visuelle humaine et le paquet macOS renouvelé ne sont pas réalisés.

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

Le binaire release courant a été construit depuis ce worktree, installé dans `/home/moi/.local/bin/bridget`, puis les services ont été redémarrés le 2026-08-31 à 07:03:08 UTC :

- `bridget-daemon.service` : actif, PID 2263491 ;
- `bridget-ui.service` : actif, PID 2263492 ;
- SHA-256 du binaire installé : `1ebce05302baee7131369f87549f6951c09a49c4a2ba73ff25a8a30f8e32f12f`.

## Preuves automatisées

| Commande | Résultat |
|---|---|
| `node --check crates/bridget-daemon/assets/ui/app.js` | PASS |
| `node crates/bridget-daemon/assets/ui/app.js` | PASS - 95 tests, 0 échec |
| `cargo fmt --check` | PASS |
| `cargo test -p bridget-daemon spec_080 --lib` | PASS - 2 tests ciblés |
| `cargo build --release -p bridget-daemon` | PASS |
| `node --check apps/bridget-desktop/ui/app.js` | PASS |
| `cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml panels` | PASS - 1 test ciblé |
| `git diff --check` | PASS |

Le test Desktop signale un avertissement préexistant : import `Path` inutilisé dans `preferences_store.rs`. Il ne provient pas des changements de finition et ne bloque pas les tests ciblés.

## Limites explicites avant clôture

- Le paquet `/Applications/Bridget.app` n'est pas encore reconstruit et remplacé. Le raccourci direct de réglages par profil ne doit donc pas être annoncé comme installé.
- La validation humaine de la vue Typographie après le portage T3 reste requise. Aucun résultat de clic ou de capture externe ambiguë n'est compté comme acceptation.
- Les coûts restent indisponibles sans tarifs versionnés. Les filtres avancés, la courbe quotidienne et les autres réglages serveur écrits restent des tâches non cochées dans `tasks.md`.
