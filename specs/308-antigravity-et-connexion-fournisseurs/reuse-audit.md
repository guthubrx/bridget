# Recherche préalable et audit de réutilisation - Spec 308

## Sources consultées (Constitution Article IX)
- Base locale `/Users/moi/.speckit/research/` (`01-ai-agents-agentic-ai.md`, `04-architectures-patterns.md`, `06-security-compliance.md`).
- Documentation officielle Google Antigravity CLI : `https://antigravity.google/docs/cli/reference.md`, `https://antigravity.google/docs/install.md`, script officiel `https://antigravity.google/cli/install.sh`.
- Inspection directe du binaire `/Users/moi/.local/bin/agy` (`1.5.13`) : `agy --help`, `agy acp --help`, `agy models`.
- Inspection du support ACP Antigravity dans `/Users/moi/11.Repositories/t3code/apps/server/src/provider/acp/AntigravityAcpSupport.ts`.
- Inspection des CLI natifs : `/opt/homebrew/bin/codex login --help`, `/Users/moi/.local/bin/claude auth login --help`, et `codex login --help` sur `cartae`.

## Signaux d'alerte identifiés
1. **`clientInfo` ACP** : `agy acp` n'expose les modèles non-Gemini (Claude 5.5, GPT-OSS) qu'aux clients reconnus (`clientInfo: { name: "zed", version: "0.0.0" }`).
2. **Authentification ACP explicite** : `agy acp` attend `authenticate` avec `methodId = "oauth-personal"` avant `session/new`.
3. **Auth SSH distante** :
   - `agy acp --login` sur SSH détecte automatiquement la session distante et affiche une URL OAuth + demande un code alphanumérique.
   - `codex login` utilise un callback localhost (`127.0.0.1:1455`) : le tunnel SSH `-L 1455:127.0.0.1:1455` permet d'ouvrir l'URL sur le Mac et de renvoyer le callback directement vers le serveur Linux distant.
   - `claude auth login` affiche une URL OAuth + demande le code de retour ou utilise un callback local.
4. **Isolation des comptes** : ne jamais copier ni stocker de secret dans Bridget ; isoler chaque compte uniquement via `CLAUDE_CONFIG_DIR`, `CODEX_HOME` et `--gemini-dir` (`antigravity_home`).

## Audit de réutilisation (`APPROVED`)

- [x] **Transport ACP existant réutilisé** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/acp.rs` est étendu sans créer un second client ACP.
- [x] **Gestionnaire de comptes existant réutilisé** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/provider_accounts.rs` est étendu pour `antigravity` et les sous-commandes `status`, `login`, `ui`.
- [x] **Serveur HTTP existant réutilisé** : `axum` (déjà dans `Cargo.toml` pour `mobile/http.rs`) sert l'interface locale `bridget accounts ui` sans nouvelle dépendance crate.
- [x] **PTY existant réutilisé** : `bridget_transport::pty` (déjà présent) ou PTY Unix standard pilote les flux interactifs de connexion sans nouvelle dépendance.
- [x] **Compatibilité client existante réutilisée** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/client_compatibility.rs` est étendu pour `agy`.
