# Spécification 311 — Console locale Bridget : Paramètres, Machines et déploiement des clients CLI

- **Statut** : En cours d'implémentation sur `main`
- **Suite de** : `specs/310-console-locale-comptes-lot-1a0/spec.md` (Lot 1a-0 + Lot 1a-1)
- **Décision utilisateur** : demande du 22 novembre 2026 — livrer à la suite l'onglet **Paramètres**, l'onglet **Machines**, et le déploiement contrôlé des principaux clients CLI (`Claude`, `Codex`, `Antigravity`, `Cursor`, `Gemini`).

## 1. Principes communs

1. **Même garde de session durcie** : toutes les nouvelles routes `/api/settings*`, `/api/machines*` et `/api/clients/*` passent par `check_private_auth` (`Host` exact, `Origin` exact sur tous les `POST`, cookie de port `HttpOnly; SameSite=Strict` + `x-bridget-token`, TTL de session, `Cache-Control: no-store`, `Referrer-Policy: no-referrer`).
2. **Aperçu avant action sensible** : toute modification de registre (`routing_policy`), toute installation/mise à jour de client CLI, et toute réparation de machine affiche sa cible et la commande/portée exacte avant exécution.
3. **Catalogue fermé** : aucune commande arbitraire ne peut être envoyée par le navigateur. Les clients CLI installables appartiennent à une liste fermée (`claude`, `codex`, `antigravity`, `cursor`, `gemini`) alignée sur `scripts/install.sh` et `specs/308`.
4. **Aucun arrêt de T3** : aucune action de réparation ou de mise à jour ne redémarre `bridget t3 serve`.

## 2. Onglet « Paramètres » (Lot A)

- Lecture et édition atomique (`0600`) de `automanaged.json` (`cli_updates`, `model_sync`, `interval_secs` borné entre 900 s et 604800 s) avec propagation aux machines joignables.
- Synchronisation manuelle de la flotte (`POST /api/settings/sync`) avec retour par machine.
- Lecture de la politique de routage (`routing_policy`) et édition en 2 temps (`preview` + `apply` liés par `preview_id` + SHA-256 + verrou + sauvegarde automatique + relecture disque) sur le registre maître local attesté (`routing_policy.enabled`, `machine_policies.<m>.enabled`, `machine_policies.<m>.compilation_capacity`, `connections[].enabled`).

## 3. Onglet « Machines & Clients CLI » (Lot B)

- Inventaire parallèle de toutes les machines connues (registre + `~/.ssh/config`) : joignabilité SSH, OS/arch, version Bridget, état du worker, et état des 5 clients CLI (`claude`, `codex`, `antigravity`, `cursor`, `gemini`).
- Ajout / mise à jour d'un host dans `~/.ssh/config` (`0600`, sauvegarde préalable, validation stricte alias/hôte/user/port, option IPv4 `AddressFamily inet`) avec test SSH immédiat et guidage clé SSH si accès refusé.
- Réparations ciblées sur une machine joignable (`sync_bridget` si même OS/arch, `fix_codex_link` sur macOS, `restart_worker`).
- Déploiement ou mise à jour contrôlée d'un des 5 clients CLI officiels sur la machine locale ou une machine SSH joignable, avec vérification post-installation (`--version`) et journal technique.

## 4. Critères d'acceptation vérifiables

1. Toutes les routes `/api/settings*`, `/api/machines*`, `/api/clients/*` refusent `401` sans `x-bridget-token` et `403` sans `Origin` valide.
2. Mise à jour `automanaged` validée et rejet des intervalles hors bornes (`400`).
3. Preview + apply d'un réglage de routage (`compilation_capacity`, `machine_policies.<m>.enabled`) sur registre local attesté avec relecture disque et conflit `409` si modifié entre-temps ; refus `403` sur registre non attesté / worker.
4. Ajout/mise à jour d'un bloc `Host` dans un `ssh_config` temporaire en `0600`, rejet d'un alias/hôte/user contenant des caractères d'injection (`400`).
5. Validation stricte du catalogue fermé des 5 clients CLI : tout client inconnu est refusé `400`.
