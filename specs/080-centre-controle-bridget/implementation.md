# Implémentation - SPEC-080 Centre de contrôle Bridget

Date: 2026-08-31
Statut: implémentation partielle, non déployée

## Livré dans le worktree de session

- Le panneau distant a une roue en bas de sa barre d'agents. Elle ouvre le centre du serveur relié par ce tunnel et ne prétend jamais gérer un autre serveur.
- `GET /v1/control/settings` publie le catalogue fermé et la portée des catégories. La mutation disponible est limitée à `project_roots.allowed_roots`.
- `POST /v1/control/settings/preview` ne persiste rien. `POST /v1/control/settings/apply` vérifie la génération et conserve le dernier reçu de commande dans l'écriture atomique de la politique.
- `GET /v1/control/usage` retourne les jetons réellement observés par fournisseur et modèle lorsque ces dimensions sont attestées au moment du fait. Son coût est explicitement non configuré tant qu'aucune grille tarifaire datée n'existe.
- Bridget Desktop conserve séparément ses préférences locales dans `preferences.json`: nom, thème, fuseau et taille de police. Le code de ces commandes Tauri ne référence ni tunnel, ni profil de serveur, ni jeton relay.

## Preuves exécutées

| Commande | Résultat |
|---|---|
| `node crates/bridget-daemon/assets/ui/app.js` | PASS - 94 tests, 0 échec |
| `node --check apps/bridget-desktop/ui/app.js` | PASS |
| `git diff --check` | PASS |
| `cargo fmt --check` et tests Rust ciblés | BLOQUÉ - `cargo`, `rustc` et `rustfmt` ne sont installés ni sur la machine de travail ni sur le serveur K3s |

## Limites à traiter avant livraison complète

- Les tests Rust ajoutés ne sont pas exécutés et aucun build Tauri ou daemon ne peut être attesté sans chaîne Rust.
- Le reçu de réglage couvre le rejeu immédiat de la dernière mutation. Un registre de reçus historique reste à implémenter pour la durabilité complète de FR-8015.
- Il n'existe pas encore de tarification datée, de courbe journalière, de filtres par projet, ni de coût API estimé.
- Mises à jour et diagnostics sont seulement signalés comme non configurés.
- La vérification manuelle avec un serveur réellement enregistré n'a pas été effectuée, et aucun déploiement, redémarrage ou installation n'a été tenté.

## Interdiction explicite

Cette session ne déploie pas Bridget, n'installe aucun paquet, ne redémarre aucun serveur et n'applique aucune maintenance distante.
