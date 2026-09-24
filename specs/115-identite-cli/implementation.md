# Journal 115 — Un fil T3 peut envoyer par la ligne de commande

- **Base** : main `bedaa208` — **Date** : 2026-09-24 — **Statut** : In Progress

## Diagnostic (données réelles)
- Transcript du coordinateur `opus_city_ai` : les 19 outils `mcp__bridget__*` ajoutés le 23/09 à
  08:18 et 16:38 UTC sont retirés le 24/09 à 02:29:44 UTC, la seconde où T3 relance son processus
  Claude. Le serveur n'est pas tombé : le nouveau processus ne l'a jamais reçu.
- Lignes de commande des processus Claude de T3 : `--mcp-config` ne contient que `t3-code`. Aucun
  `bridget mcp` sous un Claude de T3 ; les deux présents sont enfants d'app-servers Codex.
- `~/.claude.json` : aucun `mcpServers`, ni dans la sauvegarde manuelle du 23/09 07:49, ni après.
- Marqueurs : les quatre processus Claude de T3 portent un marqueur valide.
- Script de rejeu (`/tmp/forensic-identite/debug_identite.py`) sur la filiation du fil `bdget` :
  marqueur privé, naissance identique au microseconde, fichier de nom lisible et exact, preuve
  présente. Tout passe : la filiation n'était pas en cause.
- Lecture de `cmd_send` : la connexion passe par `send_control_to_daemon_at`, qui résout par
  filiation ; le message est signé par `current_agent_id`, qui ne lit que les variables des
  lanceurs. Deux preuves différentes pour une même opération.

## Correction (`cli.rs`)
`resolve_cli_agent_id` reçoit en troisième source une fermeture paresseuse sur
`resolve_current_identity`, consultée seulement si fichier et variable sont absents. Le message
est désormais signé par la même preuve que la connexion. Six usages corrigés d'un coup : `send`,
`reply`, `requests`, et les crochets `claude-runtime` et `claude-statusline`.

## Vérifications
- Tests : cas historiques inchangés ; nouveau test couvrant la filiation, sa normalisation, le refus
  d'un nom non UUID, et la priorité des lanceurs (la filiation n'est pas consultée).
- Contre le daemon réel, depuis le fil `bdget` :
  - `requests` : ancien binaire « Aucune demande suivie » ; nouveau binaire, les vraies demandes
    de l'agent, dont la contre-revue envoyée à `evols-t3` le 19/09.
  - `send` vers un identifiant inexistant : ancien refusé sur l'identité ; nouveau franchit le
    contrôle et n'échoue que sur le routage (« agent introuvable »), sans trace dans le journal.
- fmt OK ; clippy `-D warnings` OK ; recette complète **1533 réussis, 0 échec, 52 ignorés**.

## Serveur MCP de Claude
Rétabli hors dépôt par `claude mcp add --scope user bridget -- /Users/moi/.local/bin/bridget mcp`,
vérifié « Connected ». Il passe par le gestionnaire de configuration de Claude au lieu d'une édition
manuelle, mais reste exposé aux réécritures concurrentes de ce fichier. Depuis cette session, son
absence ne rend plus Bridget inutilisable : la ligne de commande prend le relais.
