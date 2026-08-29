# Baseline fournisseurs - SPEC-064

Relevé effectué le 2026-08-29 sur le serveur de travail, sans démarrer de
fournisseur et sans lire de contenu de conversation. Les empreintes sont des
SHA-256 du lanceur configuré, non des secrets d authentification.

| Fournisseur | Commande configurée | Version observée | SHA-256 | `provider_kind` | `execution_path` | Opération observable |
|---|---|---:|---|---|---|---|
| Codex | `/home/moi/.local/bin/codex -c model="gpt-5.6-terra" --dangerously-bypass-approvals-and-sandbox app-server` | `codex-cli 0.150.1` | `134063e133f0b4244fa3b251acf973d4fe4b4aeeacbdc135211bf480f59f1477` | `codex` | `codex_app_server` | JSON-RPC app-server : thread, turn, item et `turn/interrupt` |
| Claude | `/home/moi/.local/bin/claude --model claude-opus-5 --dangerously-skip-permissions --permission-mode bypassPermissions` | `2.1.221 (Claude Code)` | `60db8e88d42c24b5199c92cfd56ec88370c510c3789c6f364af748354f087ada` | `claude` | `claude_stream_json` | stream JSON : prompt, deltas, outils et `control_request` |
| Cursor | aucune entrée active dans `/home/moi/.config/bridget/agents.json`; définition native : `cursor-agent --model auto acp` | non résolu dans le `PATH` SSH non interactif | non disponible | `cursor` | `acp` | ACP : initialisation, prompt, autorisation et `session/cancel` |

## Résolution et garde d activation

Le registre conserve Cursor comme un fournisseur distinct :
`provider_kind=cursor` et `execution_path=acp`. Il n existe ni adaptateur
Cursor dédié, ni hypothèse qu un transport ACP soit Cursor.

Le binaire `cursor-agent` n a été trouvé ni dans le `PATH` de la session, ni
dans les emplacements de lancement usuels. Cette absence est une mesure, pas
une erreur masquée : toute activation Cursor reste refusée tant qu un chemin
exécutable, sa version et son empreinte ne sont pas relevés à nouveau.

Les deux fournisseurs effectivement configurés sont Codex et Claude. Les
chemins absolus du registre diffèrent de la définition native historique de
Codex, qui vise `/opt/homebrew/bin/codex`; cette SPEC doit donc toujours
utiliser la commande réellement configurée ci-dessus comme vérité de runtime.

## Commandes de preuve

```text
/home/moi/.local/bin/codex --version
/home/moi/.local/bin/claude --version
shasum -a 256 /home/moi/.local/bin/codex
shasum -a 256 /home/moi/.local/bin/claude
jq (projection commande/arguments/protocole seulement) /home/moi/.config/bridget/agents.json
```

Le relevé exclut explicitement les variables d environnement, les clés, les
prompts, les réponses et tout contenu utilisateur.
