# Plan d'implémentation - Spec 308

## Architecture retenue

1. **Transport ACP (`crates/bridget-transport/src/acp.rs`)** :
   - Enrichir `AcpOptions` (avec des valeurs `None` par défaut pour ne casser aucun appelant existant).
   - Dans `initialize` : envoyer `clientInfo` si défini (`zed`/`0.0.0` pour Antigravity afin d'exposer tous les modèles), puis appeler `authenticate` (`methodId`) si `auth_method_id` est défini.
   - Après `session/new` : si `session_mode` est fourni, envoyer `session/set_mode` ; si `model` ou `effort` est fourni, lire `configOptions` retourné par `session/new` et appeler `session/set_config_option` (`id = "model"`, `category = "thought_level"`) ou `session/set_model` en repli.

2. **Registre, cycle de vie et compatibilité (`crates/bridget-transport/src/protocol.rs`, `crates/bridget-daemon/src/{registry,lifecycle,wrapper,client_compatibility,provider_accounts,cli}.rs`)** :
   - Champ `antigravity_home: Option<String>` ajouté à `AgentDefinition`, `ResolvedAgentDefinition` et au calcul de digest (seulement quand `Some`, pour préserver 100 % des digests existants).
   - Moteur `"antigravity"` + protocole `"acp"` autorisé partout où `("claude", "claude_stream_json")` et `("codex", "codex_app_server")` sont vérifiés.
   - Injection automatique des arguments `--gemini-dir <dir>` et `--app-data-dir antigravity-bridget` lors du spawn d'`agy acp`, et transmission des options Antigravity (`auth_method_id = Some("oauth-personal")`, `client_info = Some(("zed", "0.0.0"))`, `session_mode = Some("yolo")` si `permissions == "allow"`).
   - `client_compatibility.rs` : ajouter `agy` -> moteur `antigravity`, version minimale `1.5.0`, parseur de version `X.Y.Z`.
   - `provider_accounts.rs` : ajouter `("antigravity", "acp")` dans `prepare_change()` et `safe_template()`, avec profil privé `provider-accounts/<id>/antigravity`.

3. **Connexion centralisée et fenêtre graphique locale (`crates/bridget-daemon/src/accounts_connect.rs`)** :
   - Nouveau module dédié, raccordé depuis `provider_accounts.rs` (`accounts status | login | ui`).
   - Découverte des connexions depuis le registre maître (`~/.cache/bridget-core/agents.json` sur `monordinateur` et `cartae`, avec mapping machine -> cible SSH lu depuis `machines` ou `~/.ssh/config`).
   - `status` : exécution parallèle (un thread par machine, timeout borné 8 s) des commandes de statut natives (`claude auth status`, `codex login status`, `agy models` ou présence de session dans `--gemini-dir`).
   - `login` : exécution directe locale ou `ssh -t` distante (avec `-L 1455:127.0.0.1:1455` pour Codex distant, détection d'URL OAuth et ouverture automatique sur macOS via `open`). Supporte `--prepare-id <id> --from <conn>` pour préparer, connecter et activer un nouveau compte isolé distant en une seule passe.
   - `ui` : serveur `axum` strictement sur `127.0.0.1:0` (port éphémère) avec jeton aléatoire `X-Bridget-Token` / `?token=...`, lançant des sessions de connexion dans un PTY (`bridget_transport::pty` ou `script` / `ssh -tt`) avec streaming des lignes vers le navigateur, extraction automatique de l'URL OAuth (bouton + ouverture auto), envoi du code saisi vers stdin du PTY, et bouton « + Nouveau compte isolé ».
   - Lanceur macOS `/Users/moi/Applications/Bridget Comptes.app` minimaliste qui exécute `/Users/moi/.local/bin/bridget accounts ui`.
