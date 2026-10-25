# Journal 115 — Un fil adaptateur de bureau peut envoyer par la ligne de commande

- **Base** : main `bedaa208` — **Date** : 2026-09-24 — **Statut** : Implemented, livré le 2026-09-24 07:02

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/115-identite-cli/implementation.md#extrait-1).

## Correction (`cli.rs`)
`resolve_cli_agent_id` reçoit en troisième source une fermeture paresseuse sur
`resolve_current_identity`, consultée seulement si fichier et variable sont absents. Le message
est désormais signé par la même preuve que la connexion. Six usages corrigés d'un coup : `send`,
`reply`, `requests`, et les crochets `claude-runtime` et `claude-statusline`.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/115-identite-cli/implementation.md#extrait-2).

## Serveur MCP de Claude
Rétabli hors dépôt par `claude mcp add --scope user bridget -- /Users/moi/.local/bin/bridget mcp`,
vérifié « Connected ». Il passe par le gestionnaire de configuration de Claude au lieu d'une édition
manuelle, mais reste exposé aux réécritures concurrentes de ce fichier. Depuis cette session, son
absence ne rend plus Bridget inutilisable : la ligne de commande prend le relais.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/115-identite-cli/implementation.md#extrait-3).
