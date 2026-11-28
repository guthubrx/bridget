# Spécification 315 : Rechargement à chaud de la façade `bridget mcp`

- Statut : En cours d'implémentation
- Date : 2026-11-28
- Périmètre : `crates/bridget-daemon/src/mcp.rs`, `crates/bridget-daemon/src/mcp_hot_reload.rs`, tests unitaires et d'intégration, documentation utilisateur (`skills/bridget/SKILL.md`).

## 1. Contexte et problème

Lorsqu'une nouvelle version de Bridget est déployée sur une machine (via `scripts/deploy-fleet.py` qui bascule atomiquement `$HOME/.local/bin/bridget` vers `$HOME/.local/lib/bridget-communication/<build>/bin/bridget`), les services daemon et worker redémarrent sur le nouveau binaire. En revanche, les processus `bridget mcp` déjà ouverts par T3 Code, Claude Code, Codex ou Antigravity restent exécutés sur l'ancienne image binaire tant que la session ou T3 n'est pas redémarrée.

Deux conséquences concrètes en découlent :

1. Les sessions ouvertes ne bénéficient pas des nouveaux outils MCP, des nouveaux paramètres, ni des corrections livrées dans la façade MCP.
2. Le serveur MCP n'annonce pas la capacité `capabilities.tools.listChanged: true` et n'émet pas de notification `notifications/tools/list_changed`, ce qui empêche les clients MCP conformes de rafraîchir automatiquement leur catalogue d'outils en cours de session.

## 2. Objectif

Permettre à chaque processus `bridget mcp` actif sur Unix (macOS et Linux) de détecter automatiquement le changement du binaire Bridget actif, de se remplacer en place par `execv` sans perte d'état ni d'octet sur `stdin`/`stdout`, puis d'émettre `notifications/tools/list_changed` pour que les sessions existantes bénéficient immédiatement de la nouvelle version sans redémarrage de T3 Code ni fermeture de fil.

## 3. Exigences fonctionnelles

### FR-001 : Annonce de la capacité dynamique `tools.listChanged`
- La réponse JSON-RPC à `initialize` dans `bridget mcp` doit déclarer :
  `"capabilities": { "tools": { "listChanged": true } }`.
- Les schémas compacts (`BRIDGET_MCP_COMPACT_SCHEMAS=1`) restent strictement sous le plafond Antigravity (< 600 octets par outil).

### FR-002 : Détection du lien actif surveillé
- En mode normal (sans surcharge), `bridget mcp` détermine le lien symbolique actif à surveiller :
  1. Si `BRIDGET_MCP_HOT_RELOAD` vaut `0` ou `false` (insensible à la casse), la surveillance est désactivée.
  2. Si `BRIDGET_MCP_WATCH_LINK` est défini et non vide, ce chemin exact est surveillé (usage test ou installation dédiée).
  3. Sinon, si `argv[0]` est un chemin existant pointant vers un lien symbolique (ex. `~/.local/bin/bridget`), ce lien est surveillé.
  4. Sinon, si l'exécutable courant (`current_exe`) réside dans l'arborescence d'installation gérée `$HOME/.local/lib/bridget-communication/`, le lien canonique `$HOME/.local/bin/bridget` est surveillé (couvre les hôtes comme T3 Code qui résolvent le lien avant `exec`).
  5. Dans tous les autres cas (ex. binaire local `target/debug/bridget` ou `target/release/bridget`), la surveillance reste inactive afin de ne jamais basculer involontairement un environnement de développement ou de test.
- Si `T3CODE_BRIDGET_EXECUTABLE` pointe vers un fichier direct hors `$HOME/.local/lib/bridget-communication/` et qui n'est pas un lien symbolique, la surveillance reste inactive.

### FR-003 : Lecture `stdin` sans perte d'octet et surveillance en veille
- `run_stdio()` utilise un lecteur sans tampon d'avance (`UnbufferedPollLineReader`) reposant sur `poll(2)` et `read(2)` sur `STDIN_FILENO`.
- Entre deux lignes complètes, `poll(2)` se réveille périodiquement (2000 ms par défaut, configurable via `BRIDGET_MCP_HOT_RELOAD_POLL_MS` borné à `[50, 60000]` ms) pour détecter un changement du lien actif même lorsque le client est silencieux.
- Aucun octet appartenant à un message suivant n'est pré-lu en mémoire utilisateur avant le `\n` courant : un `execv` déclenché entre deux requêtes ne peut perdre aucun octet déjà écrit dans le tube noyau par le client.
- Aucun `execv` n'est déclenché au milieu d'une ligne partielle, ni si `stdin` est fermé (`POLLHUP` sans `POLLIN`), ni entre `initialize` et `notifications/initialized`.

### FR-004 : Frontière sûre de remplacement (`execv`)
- Un remplacement `execv` n'est déclenché que lorsque :
  - zéro appel `tools/call` n'est en vol (`registry.calls.is_empty()` et compteur `active_workers == 0`),
  - le `std::thread::scope` s'est terminé et a joint tous les threads workers (processus strictement mono-thread avant `execv`),
  - `stdout` et `stderr` ont été vidés (`flush`).
- Les descripteurs `0` (stdin), `1` (stdout), `2` (stderr), le PID, le PPID, la date de naissance du processus (`birth`) et les variables d'environnement métier (`BRIDGET_MCP_COMPACT_SCHEMAS`, `BRIDGET_HOME`, `HOME`, etc.) sont intégralement préservés, maintenant intacte la filiation d'identité (`mcp_identity` et `t3code_identity`).
- L'exécutable appelé par `execv` est le chemin canonique validé du nouveau binaire, avec `argv = [target_canonical, "mcp"]`, ce qui rend la nouvelle version directement visible dans `ps`.

### FR-005 : Transmission interne d'état et notification `list_changed`
- Avant `execv`, l'état minimal de session et de protection anti-boucle est sérialisé dans la variable d'environnement privée `BRIDGET_MCP_HOT_RELOAD_V1` (JSON fermé, `version: 1`, `initialize_seen`, `initialized`, `watch_link`, `reexec_window_start`, `reexec_count`, `last_reexec_at_ms`).
- La grammaire CLI fermée de `bridget mcp` (zéro argument via `validate_zero_arity_command`) n'est pas modifiée.
- Dès l'entrée mono-thread de `run_stdio()`, le nouveau processus lit et supprime immédiatement `BRIDGET_MCP_HOT_RELOAD_V1` de son environnement pour éviter toute fuite vers des sous-processus.
- Si l'état restauré indique `initialized == true`, le nouveau processus émet immédiatement une unique notification JSON-RPC 2.0 sur `stdout` :
  `{"jsonrpc":"2.0","method":"notifications/tools/list_changed"}\n`
  et la vide (`flush`) avant d'attendre la requête suivante.
- Si l'état restauré indique `initialized == false` (bascule avant handshake), aucune notification n'est émise.

### FR-006 : Validation de sécurité et garde-fous anti-boucle
- Avant d'accepter un binaire cible pour `execv`, le processus vérifie :
  1. L'identité de fichier `(dev, ino)` diffère de celle du binaire en cours d'exécution.
  2. Le fichier cible résolu est un fichier régulier, appartient à `geteuid()`, possède le bit d'exécution propriétaire (`0o100`), et n'est ni group-writable ni world-writable (`mode & 0o022 == 0`).
  3. Le répertoire parent du lien surveillé et celui du binaire cible appartiennent à `geteuid()` et ont `mode & 0o022 == 0`.
  4. En mode normal (sans `BRIDGET_MCP_WATCH_LINK`), si le binaire courant provient de `$HOME/.local/lib/bridget-communication/`, le binaire cible doit également résider dans cette même arborescence canonique.
- Anti-boucle :
  - Limite maximale de 3 `execv` par fenêtre glissante de 60 secondes (`MAX_REEXEC_PER_WINDOW = 3`, `REEXEC_WINDOW_SECS = 60`).
  - Délai minimal de 2000 ms entre deux `execv` en production (configurable en test via `BRIDGET_MCP_HOT_RELOAD_MIN_INTERVAL_MS`).
  - Si `execv` échoue ou si la cible est invalide, son `(dev, ino)` est mémorisé comme en échec jusqu'au prochain changement, un diagnostic est écrit sur `stderr`, la variable interne `BRIDGET_MCP_HOT_RELOAD_V1` est retirée, et le processus continue de servir normalement la session MCP courante sans jamais fermer le tube.

## 4. Critères d'acceptation (Gherkin)

```gherkin
Feature: Rechargement à chaud de bridget mcp

  Scenario: Annonce de la capacité listChanged lors de initialize
    Given une session bridget mcp fraîche
    When le client envoie la requête JSON-RPC initialize
    Then la réponse contient result.capabilities.tools.listChanged == true

  Scenario: Bascule transparente à l'état calme avec notification list_changed
    Given un processus bridget mcp lancé via un lien symbolique vers un binaire v1
    And la session a terminé initialize et notifications/initialized
    When le lien symbolique bascule atomiquement vers un binaire v2 valide
    Then le processus se remplace en place avec le même PID
    And le client reçoit {"jsonrpc":"2.0","method":"notifications/tools/list_changed"} sur stdout
    And une requête tools/list ou ping suivante reçoit une réponse valide sur le même tube

  Scenario: Aucun octet stdin perdu sur des requêtes pipelinées
    Given un processus bridget mcp initialisé
    When deux requêtes JSON-RPC sont écrites simultanément sur stdin
    Then les deux requêtes sont traitées dans l'ordre sans perte d'octet

  Scenario: Refus d'un binaire non sûr ou d'une boucle rapide
    Given un lien pointant vers un fichier world-writable ou dépassant le plafond de 3 execv/60s
    When le contrôleur de hot-reload évalue la cible
    Then aucun execv n'est exécuté et la session continue normalement sur le binaire courant
```
