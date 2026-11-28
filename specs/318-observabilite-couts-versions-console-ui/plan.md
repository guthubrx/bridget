# Plan 318 : Observabilité coûts/versions + Console UI

1. **Logos moteurs UI** : créer `crates/bridget-daemon/assets/console/logos.js`, exposer `/assets/logos.js` dans `console_ui.rs`, intégrer dans `usage.js`, `accounts.js`, `machines.js` et `styles.css`.
2. **Détection CLI hors registre dans Comptes** : étendre `accounts_connect.rs` (`gemini` + détection des 5 CLI officiels installés hors `agents.json` dans `probe_machine` + probing des machines sans entrée `agents.json`) et `console_login.rs` (résolution `detected-<engine>-<machine>` pour le login interactif).
3. **Vraie détection de version CLI + Tout mettre à jour** : ajouter un cache 15 min des dernières versions amont (`registry.npmjs.org` pour `codex`, `claude`, `gemini` ; honnête `unknown_upstream` pour `antigravity`/`cursor`) + comparateur semver dans `console_admin.rs`, mise en forme claire et file de mise à jour groupée dans `machines.js` + `app.js`.
4. **Suivi global coûts/versions toutes missions** :
   - Estampiller `bridget_build_id` sur `delegation::Task` (`delegation.rs` + `daemon/native_delegation.rs`).
   - Créer `bridget_build_history` dans `convergence_metrics.rs` et enregistrer le build courant au démarrage du daemon (`daemon.rs`).
   - Ajouter `estimate_tokens_cost_usd` et l'agrégation globale `fleet_tasks_by_build` / `build_history` / `fleet_compare` dans `convergence_metrics.rs` + `cli.rs` (`bridget metrics`).
5. **Tests, revue adverse Claude, commit, build macOS/Linux, déploiement 371 et ménage**.
