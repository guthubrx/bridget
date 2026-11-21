# SPEC-310 — Console locale Bridget (Lot 1a-0)

Date : 2026-11-21
Référence de cadrage : `/tmp/concile-bridget-console-20261121/cadrage-r5.md` et `DECISION.md`
Branche unique : `main` (`197bfca2`)

## 1. Objectif du Lot 1a-0

Remplacer l'écran web local actuel de `bridget accounts ui` par une console locale sobre et professionnelle fondée sur la maquette 2 (Graphite sombre + Atelier clair + mode Système sans flash), dotée d'un durcissement HTTP complet, d'un routeur minimal fermant les anciennes routes non durcies, et d'une première écriture réelle sous verrou (`enable` / `disable` sur registre local/maître attesté uniquement).

## 2. Décisions d'architecture (ADR)

- **ADR-310-1 (Zéro Node / `include_str!`)** : les sources frontend (`index.html`, `styles.css`, `app.js`, `api.js`, `views/accounts.js`) vivent dans `crates/bridget-daemon/assets/console/` et sont embarquées dans le binaire Rust par `include_str!`. Réexamen d'un bundler uniquement si le JS dépasse 2 000 lignes.
- **ADR-310-2 (Point d'entrée CLI)** : `bridget accounts ui` reste le seul point d'entrée du Lot 1a-0, respectant le retrait volontaire de `bridget ui` (`crates/bridget-daemon/src/cli.rs:341-345`).

## 3. Plafonds mesurables (avant premier commit)

- **UI (HTML + CSS + JS, tests exclus)** : <= 800 lignes cumulées.
- **Rust (nouveau module `console_ui.rs` + exports `provider_accounts.rs` + raccord `accounts_connect.rs`, tests exclus)** : <= 950 lignes.
- **Délai** : 1 semaine calendaire à compter du premier commit.

## 4. Contrats de sécurité, d'autorité et de persistance

1. **Amorçage de session et isolement inter-ports (RFC 6265 §8.5 + HTML Origin)** :
   - Réponses publiques d'amorçage strictement limitées au shell HTML vide de données privées (`GET /`) et aux ressources statiques (`GET /assets/*`), sous contrôle strict de `Host == 127.0.0.1:<port>`.
   - `GET /?token=<launch_token>` consomme `launch_token` une seule fois, pose le cookie de session propre au port (`bridget_console_sid_<port>=<sid>; HttpOnly; SameSite=Strict; Path=/`) et redirige (`303 See Other`) vers `/#bt=<client_token>` (`Cache-Control: no-store`, `Referrer-Policy: no-referrer`).
   - Le script d'amorçage stocke `<client_token>` dans `sessionStorage` (cloisonné par origine `http://127.0.0.1:<port>`) et nettoie l'URL (`history.replaceState(null, "", "/")`).
   - Toute route privée (`/api/*`) exige : `Host` exact + cookie de session non expiré + en-tête `x-bridget-token` exact. Toute mutation (`POST`) exige en plus `Origin == http://127.0.0.1:<port>`. Un cookie présenté seul (ex. capté sur un autre port) est refusé sur toute lecture privée et toute mutation.
2. **Routeur minimal Lot 1a-0** :
   - Routes enregistrées : `GET /`, `GET /assets/{styles.css,app.js,api.js,views/accounts.js}`, `GET /api/status`, `POST /api/ui-prefs`, `POST /api/accounts/preview`, `POST /api/accounts/apply`.
   - Toutes les anciennes routes différées (`/api/prepare`, `/api/login/*`, `/api/automanaged/*`) sont absentes du routeur (`404`).
3. **Thèmes sans flash (`ui-prefs.json`)** :
   - Stockage dans `$BRIDGET_HOME/ui-prefs.json` ou `~/.cache/bridget-core/ui-prefs.json`, mode `0600`, écriture atomique + `fsync`.
   - Repli non bloquant sur `system` avec journalisation si absent ou invalide.
   - Injection serveur sur `<html data-theme-pref="...">` + résolution `<head>` via `matchMedia` avant premier rendu + sélecteur `role="radiogroup"`.
4. **Source native d'autorité et écritures `enable` / `disable`** :
   - Résolution serveur explicite : seul un registre local non-worker (`!worker-*.json`, `!~/.config/bridget/agents.json`) correspondant au chemin attesté `crate::registry::config_path()` et dont le `machine_scope` correspond à l'hôte local est autorisé en écriture (`Maître (registre)`).
   - Tout registre worker (y compris `worker-monordinateur-agents.json` sur le même hôte Mac), tout maître distant et tout chemin `--registry` non attesté sont maintenus en lecture seule (`Worker <id> · Lecture seule`, `Maître distant · Lecture seule`, `Propriétaire indéterminé · Lecture seule`).
   - Deux temps stricts via `provider_accounts` (sans PTY) : aperçu borné lié à la session, au registre, à l'action, à l'identifiant et au SHA-256 ; confirmation sous `project-preparation.lock`, vérification SHA-256, `fsync` sauvegarde + remplacement atomique, relecture disque `accounts[].enabled` + `registry_sha256`.
   - État retourné honnête : `Enregistré et relu — prise en compte à confirmer` (pas de modification du protocole daemon en Lot 1a-0).

## 5. Critères de réception automatisés

- Échange unique du jeton de lancement + redirection `/#bt=...`, refus au second usage.
- Expiration de session, refus `Host` étranger, refus `Origin` absent ou étranger sur `POST`.
- Scénario inter-ports sur deux instances réelles : cookie présenté seul refusé sur `GET /api/status` et `POST /api/ui-prefs`.
- Fermeture effective des anciennes routes (`/api/prepare`, `/api/login/input`, `/api/automanaged/toggle`, `/api/automanaged/sync`) -> `404`.
- Persistance du thème dans `ui-prefs.json` (`0600`), réouverture sur un second port éphémère sans flash, repli propre sur `system` si fichier corrompu.
- Aperçu + confirmation `enable`/`disable`, conflit SHA-256 (`Conflit : rechargez l'aperçu`), registre worker local et `--registry` non attesté refusés en écriture.
- Machine connue injoignable conservée visible avec statut `Machine injoignable`.
