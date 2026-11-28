# Plan technique 315 : Rechargement à chaud de la façade `bridget mcp`

## 1. Architecture cible

### Nouveau module `crates/bridget-daemon/src/mcp_hot_reload.rs`
Isoler toute la logique OS / sécurité / `poll(2)` / `execv` hors de `mcp.rs` pour garder `mcp.rs` lisible et testable unitairement :

1. **Structures de données** :
   - `FileFingerprint { dev: u64, ino: u64 }` (extrait de `std::os::unix::fs::MetadataExt`).
   - `HotReloadState` (sérialisé en JSON dans `BRIDGET_MCP_HOT_RELOAD_V1`, `#[serde(deny_unknown_fields)]`) :
     - `version: u8` (`1`)
     - `initialize_seen: bool`
     - `initialized: bool`
     - `watch_link: Option<PathBuf>`
     - `reexec_window_start: i64`
     - `reexec_count: u32`
     - `last_reexec_at_ms: i64`
   - `HotReloadPlan { target_binary: PathBuf, target_fingerprint: FileFingerprint, state: HotReloadState }`
   - `HotReloadController` :
     - Conserve `enabled`, `watch_link`, `managed_root: Option<PathBuf>`, `current_fingerprint`, `poll_interval`, `min_interval`, compteurs de fenêtre (`reexec_window_start`, `reexec_count`, `last_reexec_at_ms`), `failed_fingerprint: Option<FileFingerprint>`.
     - Méthodes :
       - `take_handoff_state() -> Option<HotReloadState>` (lit et supprime immédiatement `BRIDGET_MCP_HOT_RELOAD_V1` au démarrage mono-thread).
       - `from_process_env(handoff: Option<&HotReloadState>) -> Self` (résout `watch_link` selon FR-002).
       - `poll_timeout_ms(&self) -> i32`.
       - `check_reload(&mut self, session_initialize_seen: bool, session_initialized: bool) -> Option<HotReloadPlan>` (valide FR-006 et anti-boucle).
       - `mark_failed(&mut self, fp: FileFingerprint)`.
       - `exec_replace(plan: &HotReloadPlan) -> io::Error` (pose `BRIDGET_MCP_HOT_RELOAD_V1`, prépare `CString` de `target_binary` et `argv = [target_binary, "mcp"]`, appelle `libc::execv`; si retour en erreur, retire immédiatement `BRIDGET_MCP_HOT_RELOAD_V1` et retourne `io::Error::last_os_error()`).

2. **Lecteur `stdin` sans perte : `PollLineReader`** :
   - Encapsule un descripteur Unix (`RawFd`, `STDIN_FILENO` en production, extrémité lecture d'un pipe en test).
   - Trait interne `LineSource` avec méthode :
     ```rust
     enum ReadLineEvent {
         Line(String),
         IdleTimeout,
         Eof,
     }
     ```
   - Implémentation pour tout `R: BufRead` (préserve 100 % la signature publique `pub fn serve<R: BufRead, W: Write + Send>` et tous les tests existants qui passent des buffers mémoire !).
   - Implémentation `PollLineReader` pour `run_stdio()` :
     - Appelle `libc::poll` sur `fd` avec `POLLIN` et timeout `poll_timeout_ms` (gère `EINTR`).
     - Si timeout (`0`) : renvoie `Ok(ReadLineEvent::IdleTimeout)`.
     - Si `POLLHUP`/`POLLERR` sans `POLLIN` : tente un `read(fd, 1)` ; si `0`, renvoie `Ok(ReadLineEvent::Eof)`.
     - Si `POLLIN` : lit octet par octet jusqu'à `\n` (ou EOF, ou plafond `MAX_MCP_LINE_BYTES = 16 * 1024 * 1024`).
     - Méthode auxiliaire `stdin_has_hangup_only(&self) -> bool` (`poll` timeout 0) pour éviter tout `execv` inutile sur un pipe déjà fermé.

### Modifications minimales dans `crates/bridget-daemon/src/mcp.rs`
1. `initialize_result()` :
   `"capabilities": { "tools": { "listChanged": true } }`.
2. `run_stdio()` :
   - Prend le `HotReloadState` via `mcp_hot_reload::take_handoff_state()` (mono-thread).
   - Initialise `Session` depuis le handoff (ou `Session::default()`).
   - Construit `HotReloadController::from_process_env(handoff.as_ref())`.
   - Si le handoff existe et `session.initialized`, émet immédiatement `{"jsonrpc":"2.0","method":"notifications/tools/list_changed"}\n` sur `stdout` et `flush()`.
   - Boucle externe :
     - Lance `serve_with_source(...)` avec `PollLineReader::stdin()` et `&mut controller`.
     - Si `ServeOutcome::Eof` -> retour `Ok(())`.
     - Si `ServeOutcome::HotReload(plan)` -> tous les threads du `std::thread::scope` sont déjà joints et `stdout` déjà flushé ! On appelle `HotReloadController::exec_replace(&plan)`. Si `execv` échoue, on loggue sur `stderr`, on appelle `controller.mark_failed(plan.target_fingerprint)`, et la boucle continue avec la même `Session` sans couper la connexion !
3. Suivi exact des threads workers actifs (`active_workers: Arc<AtomicUsize>`) :
   - Incrémenté juste avant `scope.spawn(...)`, décrémenté par un `WorkerActiveGuard` (créé en premier dans la closure, donc droppé en dernier, APRÈS `write_shared_response` et `acknowledge_user_input_receipt`).
   - Condition stricte pour déclencher le rechargement :
     `active_workers.load(Ordering::Acquire) == 0 && registry.lock().recover_poison().calls.is_empty()` ET `!input.stdin_has_hangup_only()`.
   - Vérification déclenchée à deux moments propres :
     - Sur `ReadLineEvent::IdleTimeout` (quand le client est calme).
     - Juste avant de lire une nouvelle ligne (uniquement si `controller.check_due()`), de sorte qu'une bascule détectée pendant qu'un appel était en vol s'exécute dès que l'appel se termine ou au prochain créneau calme.

### Vérification de rétrocompatibilité
- `pub fn serve<R: BufRead, W: Write + Send>` garde exactement sa signature et son comportement (hot-reload inactif sur `BufRead` mémoire).
- `validate_zero_arity_command("mcp", ...)` inchangé.
- Tous les schémas compacts MCP inchangés (< 600 octets).
- Seul test existant à mettre à jour : `matrice_fr009_couvre_les_quinze_cas` dans `mcp_tests.rs:2365` pour refléter `json!({ "tools": { "listChanged": true } })`.

## 2. Stratégie de tests (TDD)

1. **Tests unitaires dans `mcp_hot_reload.rs`** :
   - Validation de cible sûre (refuse world-writable `0o777`, group-writable `0o775`, non-exécutable `0o600`, fichier inexistant, même `(dev, ino)`).
   - Validation du répertoire parent (refuse répertoire parent `0o777`).
   - Résolution `watch_link` : opt-out `BRIDGET_MCP_HOT_RELOAD=0`, `BRIDGET_MCP_WATCH_LINK`, symlink `argv[0]`, binaire dans `$HOME/.local/lib/bridget-communication/<build>/bin/bridget`, et binaire dev hors arborescence (doit rester désactivé).
   - Garde anti-boucle : max 3 re-exec / 60 s, respect `min_interval`, marquage `failed_fingerprint` ignoré jusqu'au changement de cible.
   - Sérialisation / désérialisation `HotReloadState` + suppression immédiate de `BRIDGET_MCP_HOT_RELOAD_V1`.
   - `PollLineReader` sur vrai pipe Unix : timeout idle, lecture exacte de deux lignes pipelinées sans sur-lecture (vérification qu'après lecture de la ligne 1, la ligne 2 est encore lisible directement sur le `RawFd` !), EOF propre, gestion ligne sans `\n` final.
2. **Test d'intégration réel `execv` dans `crates/bridget-daemon/tests/mcp_hot_reload_315_test.rs`** :
   - Crée un répertoire privé `0700` contenant deux copies (ou hardlinks distincts -> deux fichiers distincts donc inodes distincts !) du vrai binaire `bridget`, nommés `v1/bridget` et `v2/bridget`, et un symlink `active -> v1/bridget`.
   - Lance `active mcp` avec `BRIDGET_MCP_WATCH_LINK=<active>` et `BRIDGET_MCP_HOT_RELOAD_POLL_MS=80`.
   - Envoie `initialize` (vérifie `capabilities.tools.listChanged == true`), puis `notifications/initialized`, puis `ping` (vérifie la réponse).
   - Enregistre le `PID` du processus.
   - Bascule atomiquement le symlink `active -> v2/bridget` pendant que le serveur est calme.
   - Lit automatiquement `{"jsonrpc":"2.0","method":"notifications/tools/list_changed"}` sur `stdout` (sans envoyer aucune requête !).
   - Vérifie que le `PID` est identique, envoie `tools/list` et `ping`, et vérifie des réponses valides sur le même tube !
   - Teste aussi le repli sans crash si le symlink pointe vers un fichier corrompu/world-writable (la session continue de répondre à `ping`).
3. **Revue adverse indépendante** via Claude (`bridget_auto_delegate`) avant livraison finale.
