# Plan technique - Spec 309 : Comptes regroupés, modèles auto-référencés et CLI automanaged

## Architecture retenue
1. **Regroupement par vrai compte** (`crates/bridget-daemon/src/accounts_connect.rs`) :
   - Ajout des champs `covered_connections: Vec<String>` et `account_label: String` sur `ConnectionStatus` (rétrocompatible JSON).
   - Avant `probe_machine`, regrouper les `ConnectionTarget` d'une même machine par `(engine, canonical_command_or_basename, profile_dir.unwrap_or_default())`.
   - Choisir la connexion principale la plus propre (préférer celle sans suffixe historique `-haiku213` / `-terra213` / `-models62` / `-native`, sinon ordre alphabétique), sonder une seule fois par groupe, et exposer `covered_connections` dans le tableau CLI et dans chaque carte de **Bridget Comptes**.
2. **Sonde Antigravity dans `CatalogueService`** (`crates/bridget-daemon/src/provider_catalogue.rs`, `crates/bridget-daemon/src/daemon/native_provider_catalogue.rs`) :
   - Ajouter `probe_antigravity(def, source, salt, stopping)` qui exécute `agy models` (ou vérifie les identifiants isolés si `antigravity_home` est défini) dans `lifecycle::build_environment(def, source)` (variables interdites nettoyées), parse les identifiants de modèles valides (`catalogue_token`), et retourne `Observed` ou `SignInRequired`.
   - Dans `native_provider_catalogue.rs`, autoriser `matches!(connection.engine.as_str(), "codex" | "claude" | "antigravity")` et projeter les modèles annoncés d'Antigravity comme `reported_models` + `auth: "reported_present"` (ou `sign_in_required` + `reported_absent`).
3. **Auto-référencement additif des modèles & gestion `automanaged`** (`crates/bridget-daemon/src/automanaged.rs`, `crates/bridget-daemon/src/cli.rs`, `crates/bridget-daemon/src/lib.rs`) :
   - Configuration locale privée `~/.cache/bridget-core/automanaged.json` (`0600`) avec défauts `cli_updates: true`, `model_sync: true`, `interval_secs: 21600` (6 h).
   - `bridget automanaged status [--json] | on [--cli|--models] | off [--cli|--models] | sync [--cli] [--models] [--dry-run] [--registry PATH]` :
     - **Modèles** : sonde les modèles réellement annoncés par les CLI locaux (`agy models` pour Antigravity, et pour Codex lit soit `bridget delegate --catalogue --observed --json` soit `codex app-server` `model/list`), puis fusionne **de manière strictement additive** les nouveaux modèles valides dans `agents.<type>.capabilities.models` et `routing_policy.model_catalog.<connection_id>` (verrou `project-preparation.lock`, sauvegarde `.bak-automanaged`, validation `AgentRegistry::from_json`, écriture atomique `0600`).
     - En mode flotte depuis la machine locale ou `cartae`, propage aussi les nouveaux modèles découverts des workers vers le registre maître `cartae`.
     - **CLI** : exécute les commandes officielles bornées (`claude update`, `brew upgrade --cask codex` sur macOS si Homebrew sinon `codex update`, `agy update`), capture version avant/après, valide par `bridget clients check --json`, et consigne l'état dans `~/.cache/bridget-core/automanaged-state.json` (`0600`).
4. **Exécution périodique automatique dans `scripts/update.py`** (compatible Python 3.9.6) :
   - Dans `Updater.step()`, à la fin du cycle (ou toutes les `interval_secs` = 6 h), si `automanaged.json` n'a pas désactivé la fonction, appelle `self.binary automanaged sync --quiet` avec un budget de temps borné, sans jamais bloquer ni casser les mises à jour de Bridget lui-même.
   - Déployer aussi le script `scripts/update.py` mis à jour sur les 4 machines (là où `com.bridget.updates` / `bridget-updates.service` le charge).
5. **Contrôle dans Bridget Comptes** (`crates/bridget-daemon/src/accounts_connect.rs`) :
   - Endpoints locaux protégés par jeton : `GET /api/automanaged`, `POST /api/automanaged/toggle`, `POST /api/automanaged/sync`.
   - Bandeau discret en haut de **Bridget Comptes** : statut `Automanaged (modèles + CLI) : ACTIF`, interrupteur ON/OFF, dernier passage, et bouton **Synchroniser maintenant**.
