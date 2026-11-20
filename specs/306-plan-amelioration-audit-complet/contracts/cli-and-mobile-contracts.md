# Contrats CLI, MCP, Mobile et Signature (306)

## 1. Nouvelles sous-commandes et options CLI
- `bridget federation links [--json]` : retourne la liste structurée des liens de fédération actifs (remplace l'injection `BRIDGET_SENTINEL_PREPARE_SOURCE` + `exec()` dans `scripts/sentinel.py`).
- `bridget status --metrics [--json]` : enrichit le statut existant avec les compteurs `daemon::Metrics`, les quantiles de `native_tick` et l'occupation de la socket Unix.
- `bridget doctor [--json]` : diagnostic unifié (daemon, permissions 0700/0600, bases SQLite/WAL/versions, workers, mobile, métriques).
- `bridget backup <destination> [--json]` : crée une sauvegarde cohérente à chaud de `BRIDGET_HOME` avec manifeste SHA-256.
- `bridget restore --verify <source> [--json]` : vérifie les empreintes et l'intégrité SQLite dans un bac à sable temporaire sans toucher `BRIDGET_HOME` actif.
- `bridget delegate ... --dry-run [--json]` et `bridget auto-delegate ... --dry-run [--json]` : calcule le placement, les scores, quotas et autorisations sans exécution ni réservation.
- `bridget completions <bash|zsh|fish>` : écrit le script de complétion sur `stdout`.
- Variable d'environnement `BRIDGET_LOG_FORMAT=json` : active l'émission de journaux JSON structurés (`ts`, `level`, `target`, `msg`, champs de corrélation présents).

## 2. Contrats de la passerelle HTTP mobile (`/v1/...`)
Toutes les routes existantes restent 100 % rétrocompatibles et sont servies via `axum::Router`.
Extensions opt-in (soumises à une portée explicite activée côté configuration mobile ; refus `403` structuré si désactivée) :
- `POST /v1/agents/{id}/relaunch` : relance encadrée d'un agent arrêté connu du registre.
- `POST /v1/specialists/{id}/launch` : lancement d'un profil spécialiste approuvé avec paramètres bornés.
- `GET /v1/ledger/search?q=<terme>&project=<p>&limit=<n>` : recherche plein texte bornée dans l'historique mobile autorisé.
- `PUT /v1/notifications` : accepte les préférences de filtrage par `projects` (liste blanche optionnelle) et `min_urgency` (`all` | `normal` | `high`).

## 3. Contrat de signature détachée (`ssh-keygen -Y`)
- Namespace OpenSSH fixe : `bridget-release`.
- Fichier de signature : `<archive>.sig` produit par `scripts/sign-release.sh`.
- Vérification dans `scripts/install.sh` : si `BRIDGET_RELEASE_ALLOWED_SIGNERS` (ou fichier embarqué/configuré) est actif, vérifie `ssh-keygen -Y verify -f <allowed_signers> -I <identity> -n bridget-release -s <archive>.sig < <archive>` avant extraction `tar`. Refus strict si signature absente, corrompue ou clé inconnue.

## 4. Contrat `reaper` Phase 1 opt-in
- Signal `background_liveness` exposé sur chaque session suivie (activité récente / tâches actives).
- Mode Phase 0 (observation seule) conservé par défaut ; Phase 1 (nettoyage effectif) activée uniquement par opt-in explicite (`BRIDGET_REAPER_MODE=cleanup` ou flag dédié) sur les sessions prouvées inactives sans tâche en cours.
