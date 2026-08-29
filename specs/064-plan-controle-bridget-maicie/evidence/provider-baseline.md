# Baseline fournisseurs - SPEC-064

Relevé effectué le 2026-08-29 sur le serveur de travail, sans démarrer de
fournisseur et sans lire de contenu de conversation. Les empreintes sont des
SHA-256 du lanceur configuré, non des secrets d authentification.

| Fournisseur | Commande configurée | Version observée | SHA-256 | `provider_kind` | `execution_path` | Opération observable |
|---|---|---:|---|---|---|---|
| Codex | `/home/moi/.local/bin/codex -c model="gpt-5.6-terra" --dangerously-bypass-approvals-and-sandbox app-server` | `codex-cli 0.150.1` | `134063e133f0b4244fa3b251acf973d4fe4b4aeeacbdc135211bf480f59f1477` | `codex` | `codex_app_server` | JSON-RPC app-server : thread, turn, item et `turn/interrupt` |
| Claude | `/home/moi/.local/bin/claude --model claude-opus-5 --dangerously-skip-permissions --permission-mode bypassPermissions` | `2.1.221 (Claude Code)` | `60db8e88d42c24b5199c92cfd56ec88370c510c3789c6f364af748354f087ada` | `claude` | `claude_stream_json` | stream JSON : prompt, deltas, outils et `control_request` |
| Cursor | `/home/moi/.local/bin/cursor-agent --mode ask acp` | `2026.08.25-3e8eec8` | `2ccc9a8e167797641448b5e5c936f006ba137a2555f117f38c5eb76a5238a233` | `cursor` | `acp` | ACP v1 : initialisation, session, prompt et `session/cancel` |

## Résolution et garde d activation

Le registre conserve Cursor comme un fournisseur distinct :
`provider_kind=cursor` et `execution_path=acp`. Il n existe ni adaptateur
Cursor dédié, ni hypothèse qu un transport ACP soit Cursor.

Le binaire Cursor est installé sous `/home/moi/.local/bin/cursor-agent`. Il
était absent du `PATH` de la session SSH non interactive, ce qui ne constitue
pas une absence de fournisseur. La commande doit donc être déclarée avec ce
chemin absolu.

Le test isolé T070 a attesté ACP v1, une session Cursor, un tour en mode
lecture seule et l annulation `session/cancel`. Cursor reste un fournisseur
distinct par `provider_kind=cursor`, mais son chemin est le transport ACP
commun : aucun adaptateur Cursor spécialisé n est introduit.

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
